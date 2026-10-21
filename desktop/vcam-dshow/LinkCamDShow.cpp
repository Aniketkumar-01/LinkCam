// LinkCam DirectShow Virtual Camera Filter DLL
// High-performance Windows COM InprocServer32 virtual webcam reading from Local\LinkCam_Video_Shm
// Hardware H.264 MFT decoding & DirectShow pin negotiation for Zoom, OBS, Teams, Discord, Meet

#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <initguid.h>
#include <dshow.h>
#include <mfapi.h>
#include <mftransform.h>
#include <mferror.h>
#include <wmcodecdsp.h>
#include <cstdint>
#include <atomic>
#include <algorithm>
#include <vector>

#ifndef __IKsPropertySet_INTERFACE_DEFINED__
#define __IKsPropertySet_INTERFACE_DEFINED__
interface DECLSPEC_UUID("31efac30-515c-11d0-a9aa-00aa0061be93") DECLSPEC_NOVTABLE
IKsPropertySet : public IUnknown {
    virtual HRESULT STDMETHODCALLTYPE Set(REFGUID guidPropSet, DWORD dwPropID, LPVOID pInstanceData, DWORD cbInstanceData, LPVOID pPropData, DWORD cbPropData) = 0;
    virtual HRESULT STDMETHODCALLTYPE Get(REFGUID guidPropSet, DWORD dwPropID, LPVOID pInstanceData, DWORD cbInstanceData, LPVOID pPropData, DWORD cbPropData, DWORD *pcbReturned) = 0;
    virtual HRESULT STDMETHODCALLTYPE QuerySupported(REFGUID guidPropSet, DWORD dwPropID, DWORD *pTypeSupport) = 0;
};
#endif

#ifndef IID_IKsPropertySet
DEFINE_GUID(IID_IKsPropertySet,
    0x31efac30, 0x515c, 0x11d0, 0xa9, 0xaa, 0x00, 0xaa, 0x00, 0x61, 0xbe, 0x93);
#endif

#ifndef KSPROPERTY_SUPPORT_GET
#define KSPROPERTY_SUPPORT_GET 1
#endif

// CLSID_LinkCamDShowFilter: {D37E5526-92F7-4E61-A76F-B590E8F88E29}
DEFINE_GUID(CLSID_LinkCamDShowFilter,
    0xd37e5526, 0x92f7, 0x4e61, 0xa7, 0x6f, 0xb5, 0x90, 0xe8, 0xf8, 0x8e, 0x29);

// AMPROPSETID_Pin: {9B00F101-156E-11D1-B16A-00C04F02299D}
DEFINE_GUID(LinkCam_AMPROPSETID_Pin,
    0x9b00f101, 0x156e, 0x11d1, 0xb1, 0x6a, 0x00, 0xc0, 0x4f, 0x02, 0x29, 0x9d);

// PIN_CATEGORY_CAPTURE: {FB6C4281-0353-11D1-905F-0000C0CC16BA}
DEFINE_GUID(LinkCam_PIN_CATEGORY_CAPTURE,
    0xfb6c4281, 0x0353, 0x11d1, 0x90, 0x5f, 0x00, 0x00, 0xc0, 0xcc, 0x16, 0xba);

#define SHM_NAME L"Local\\LinkCam_Video_Shm"
#define SHM_EVENT_NAME L"Local\\LinkCam_Video_Event"
#define SHM_MAGIC 0x4C4E4B43 // 'LNKC'

#pragma pack(push, 1)
struct ShmVideoHeader {
    uint32_t magic;
    uint32_t version;
    uint32_t width;
    uint32_t height;
    uint32_t format; // 0 = H264 NAL, 1 = NV12, 2 = RGB32
    uint32_t sequence;
    uint64_t timestamp_us;
    uint32_t payload_size;
    uint32_t is_keyframe;
    uint32_t reserved[6];
};
#pragma pack(pop)

static HINSTANCE g_hInst = NULL;
static std::atomic<long> g_cLock(0);

// Forward declarations
class LinkCamOutputPin;
class LinkCamFilter;

// ----------------------------------------------------------------------------
// Shared Memory Consumer Helper
// ----------------------------------------------------------------------------
class ShmReader {
public:
    HANDLE hMap = NULL;
    HANDLE hEvent = NULL;
    uint8_t* pBuf = NULL;

    bool Open() {
        if (pBuf) return true;
        hMap = OpenFileMappingW(FILE_MAP_READ, FALSE, SHM_NAME);
        if (!hMap) return false;
        pBuf = (uint8_t*)MapViewOfFile(hMap, FILE_MAP_READ, 0, 0, 0);
        if (!pBuf) {
            CloseHandle(hMap);
            hMap = NULL;
            return false;
        }
        hEvent = OpenEventW(SYNCHRONIZE, FALSE, SHM_EVENT_NAME);
        return true;
    }

    void Close() {
        if (pBuf) { UnmapViewOfFile(pBuf); pBuf = NULL; }
        if (hMap) { CloseHandle(hMap); hMap = NULL; }
        if (hEvent) { CloseHandle(hEvent); hEvent = NULL; }
    }

    bool WaitForFrame(DWORD timeoutMs) {
        if (!hEvent && !Open()) return false;
        return (WaitForSingleObject(hEvent, timeoutMs) == WAIT_OBJECT_0);
    }

    const ShmVideoHeader* GetHeader() const {
        return (const ShmVideoHeader*)pBuf;
    }

    const uint8_t* GetPayload() const {
        return pBuf ? (pBuf + sizeof(ShmVideoHeader)) : NULL;
    }
};

// ----------------------------------------------------------------------------
// Windows Media Foundation Hardware H.264 MFT Decoder
// ----------------------------------------------------------------------------
class MftH264Decoder {
private:
    IMFTransform* m_pDecoder = NULL;
    DWORD m_inStreamId = 0;
    DWORD m_outStreamId = 0;
    bool m_initialized = false;
    uint32_t m_width = 1920;
    uint32_t m_height = 1080;

public:
    MftH264Decoder() {}
    ~MftH264Decoder() { Shutdown(); }

    bool Init(uint32_t width = 1920, uint32_t height = 1080) {
        if (m_initialized) return true;
        m_width = width;
        m_height = height;

        HRESULT hr = MFStartup(MF_VERSION, MFSTARTUP_NOSOCKET);
        if (FAILED(hr)) return false;

        hr = CoCreateInstance(CLSID_CMSH264DecoderMFT, NULL, CLSCTX_INPROC_SERVER,
                              IID_IMFTransform, (void**)&m_pDecoder);
        if (FAILED(hr) || !m_pDecoder) {
            return false;
        }

        // Set Input Type (H.264)
        IMFMediaType* pInType = NULL;
        hr = MFCreateMediaType(&pInType);
        if (SUCCEEDED(hr) && pInType) {
            pInType->SetGUID(MF_MT_MAJOR_TYPE, MFMediaType_Video);
            pInType->SetGUID(MF_MT_SUBTYPE, MFVideoFormat_H264);
            MFSetAttributeSize(pInType, MF_MT_FRAME_SIZE, width, height);
            MFSetAttributeRatio(pInType, MF_MT_FRAME_RATE, 60, 1);
            pInType->SetUINT32(MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive);

            hr = m_pDecoder->SetInputType(m_inStreamId, pInType, 0);
            pInType->Release();
        }
        if (FAILED(hr)) {
            Shutdown();
            return false;
        }

        // Set Output Type (NV12)
        IMFMediaType* pOutType = NULL;
        hr = MFCreateMediaType(&pOutType);
        if (SUCCEEDED(hr) && pOutType) {
            pOutType->SetGUID(MF_MT_MAJOR_TYPE, MFMediaType_Video);
            pOutType->SetGUID(MF_MT_SUBTYPE, MFVideoFormat_NV12);
            MFSetAttributeSize(pOutType, MF_MT_FRAME_SIZE, width, height);
            MFSetAttributeRatio(pOutType, MF_MT_FRAME_RATE, 60, 1);
            pOutType->SetUINT32(MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive);

            hr = m_pDecoder->SetOutputType(m_outStreamId, pOutType, 0);
            pOutType->Release();
        }
        if (FAILED(hr)) {
            Shutdown();
            return false;
        }

        m_pDecoder->ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0);
        m_pDecoder->ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0);
        m_pDecoder->ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0);

        m_initialized = true;
        return true;
    }

    void Shutdown() {
        if (m_pDecoder) {
            m_pDecoder->ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0);
            m_pDecoder->ProcessMessage(MFT_MESSAGE_NOTIFY_END_STREAMING, 0);
            m_pDecoder->Release();
            m_pDecoder = NULL;
        }
        m_initialized = false;
    }

    bool Decode(const uint8_t* pNalData, uint32_t nalSize, uint64_t timestampUs,
                std::vector<uint8_t>& outNv12) {
        if (!m_initialized && !Init(m_width, m_height)) return false;
        if (!pNalData || nalSize == 0) return false;

        IMFSample* pSample = NULL;
        if (FAILED(MFCreateSample(&pSample)) || !pSample) return false;

        IMFMediaBuffer* pBuffer = NULL;
        if (FAILED(MFCreateMemoryBuffer(nalSize, &pBuffer)) || !pBuffer) {
            pSample->Release();
            return false;
        }

        BYTE* pDst = NULL;
        if (SUCCEEDED(pBuffer->Lock(&pDst, NULL, NULL)) && pDst) {
            memcpy(pDst, pNalData, nalSize);
            pBuffer->Unlock();
            pBuffer->SetCurrentLength(nalSize);
        }
        pSample->AddBuffer(pBuffer);
        pBuffer->Release();

        LONGLONG llTime = (LONGLONG)(timestampUs * 10);
        pSample->SetSampleTime(llTime);

        HRESULT hr = m_pDecoder->ProcessInput(m_inStreamId, pSample, 0);
        pSample->Release();
        if (FAILED(hr)) {
            return false;
        }

        MFT_OUTPUT_DATA_BUFFER outputBuffer = {0};
        outputBuffer.dwStreamID = m_outStreamId;

        IMFSample* pOutSample = NULL;
        if (FAILED(MFCreateSample(&pOutSample)) || !pOutSample) return false;

        IMFMediaBuffer* pOutMediaBuf = NULL;
        DWORD bufSize = m_width * m_height * 3 / 2;
        if (FAILED(MFCreateMemoryBuffer(bufSize, &pOutMediaBuf)) || !pOutMediaBuf) {
            pOutSample->Release();
            return false;
        }

        pOutSample->AddBuffer(pOutMediaBuf);
        pOutMediaBuf->Release();
        outputBuffer.pSample = pOutSample;

        DWORD dwStatus = 0;
        hr = m_pDecoder->ProcessOutput(0, 1, &outputBuffer, &dwStatus);
        if (SUCCEEDED(hr) && outputBuffer.pSample) {
            IMFMediaBuffer* pResBuf = NULL;
            if (SUCCEEDED(outputBuffer.pSample->ConvertToContiguousBuffer(&pResBuf)) && pResBuf) {
                BYTE* pData = NULL;
                DWORD curLen = 0;
                if (SUCCEEDED(pResBuf->Lock(&pData, NULL, &curLen)) && pData) {
                    if (outNv12.size() != curLen) {
                        outNv12.resize(curLen);
                    }
                    memcpy(outNv12.data(), pData, curLen);
                    pResBuf->Unlock();
                }
                pResBuf->Release();
            }
            outputBuffer.pSample->Release();
            return true;
        }

        if (outputBuffer.pSample) {
            outputBuffer.pSample->Release();
        }
        return false;
    }
};

// ----------------------------------------------------------------------------
// Color Conversion & Standby Frame Generator Helpers
// ----------------------------------------------------------------------------
static void Nv12ToRgb24(const uint8_t* nv12, uint8_t* rgb, int width, int height) {
    const uint8_t* yPlane = nv12;
    const uint8_t* uvPlane = nv12 + width * height;
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            int yVal = yPlane[y * width + x];
            int uvIdx = (y / 2) * width + (x & ~1);
            int uVal = uvPlane[uvIdx] - 128;
            int vVal = uvPlane[uvIdx + 1] - 128;

            int c = yVal - 16;
            int r = (298 * c + 409 * vVal + 128) >> 8;
            int g = (298 * c - 100 * uVal - 208 * vVal + 128) >> 8;
            int b = (298 * c + 516 * uVal + 128) >> 8;

            int row = height - 1 - y; // Bottom-up RGB
            int rgbIdx = (row * width + x) * 3;
            rgb[rgbIdx] = (uint8_t)std::clamp(b, 0, 255);
            rgb[rgbIdx + 1] = (uint8_t)std::clamp(g, 0, 255);
            rgb[rgbIdx + 2] = (uint8_t)std::clamp(r, 0, 255);
        }
    }
}

static void Nv12ToRgb32(const uint8_t* nv12, uint8_t* rgb, int width, int height) {
    const uint8_t* yPlane = nv12;
    const uint8_t* uvPlane = nv12 + width * height;
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            int yVal = yPlane[y * width + x];
            int uvIdx = (y / 2) * width + (x & ~1);
            int uVal = uvPlane[uvIdx] - 128;
            int vVal = uvPlane[uvIdx + 1] - 128;

            int c = yVal - 16;
            int r = (298 * c + 409 * vVal + 128) >> 8;
            int g = (298 * c - 100 * uVal - 208 * vVal + 128) >> 8;
            int b = (298 * c + 516 * uVal + 128) >> 8;

            int row = height - 1 - y;
            int rgbIdx = (row * width + x) * 4;
            rgb[rgbIdx] = (uint8_t)std::clamp(b, 0, 255);
            rgb[rgbIdx + 1] = (uint8_t)std::clamp(g, 0, 255);
            rgb[rgbIdx + 2] = (uint8_t)std::clamp(r, 0, 255);
            rgb[rgbIdx + 3] = 255;
        }
    }
}

static void RenderStandbyFrame(uint8_t* dst, int width, int height, GUID subtype, uint32_t frameCount) {
    bool isNv12 = (subtype == MEDIASUBTYPE_NV12);
    bool isRgb32 = (subtype == MEDIASUBTYPE_RGB32);

    if (isNv12) {
        memset(dst, 32, width * height); // Dark slate gray Y
        memset(dst + width * height, 128, width * height / 2); // Neutral UV

        int cx = width / 2;
        int cy = height / 2;
        int boxSize = 40;
        for (int y = cy - boxSize; y <= cy + boxSize; y++) {
            for (int x = cx - boxSize; x <= cx + boxSize; x++) {
                if (x == cx - boxSize || x == cx + boxSize || y == cy - boxSize || y == cy + boxSize ||
                    (x >= cx - 4 && x <= cx + 4 && y >= cy - 4 && y <= cy + 4)) {
                    dst[y * width + x] = 210;
                    int uvIdx = width * height + (y / 2) * width + (x & ~1);
                    dst[uvIdx] = 190;
                    dst[uvIdx + 1] = 70;
                }
            }
        }
    } else {
        int bpp = isRgb32 ? 4 : 3;
        for (int i = 0; i < width * height; i++) {
            dst[i * bpp] = 0x2a;     // B
            dst[i * bpp + 1] = 0x17; // G
            dst[i * bpp + 2] = 0x0f; // R
            if (isRgb32) dst[i * bpp + 3] = 255;
        }
        int cx = width / 2;
        int cy = height / 2;
        int boxSize = 40;
        for (int y = cy - boxSize; y <= cy + boxSize; y++) {
            for (int x = cx - boxSize; x <= cx + boxSize; x++) {
                if (x == cx - boxSize || x == cx + boxSize || y == cy - boxSize || y == cy + boxSize ||
                    (x >= cx - 4 && x <= cx + 4 && y >= cy - 4 && y <= cy + 4)) {
                    int row = height - 1 - y;
                    int idx = (row * width + x) * bpp;
                    dst[idx] = 0xff;     // B (cyan)
                    dst[idx + 1] = 0xd0; // G
                    dst[idx + 2] = 0x30; // R
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Media Type Construction & Memory Helpers
// ----------------------------------------------------------------------------
static void BuildMediaType(AM_MEDIA_TYPE* pmt, int width, int height, int fps, GUID subtype) {
    ZeroMemory(pmt, sizeof(AM_MEDIA_TYPE));
    pmt->majortype = MEDIATYPE_Video;
    pmt->subtype = subtype;
    pmt->bFixedSizeSamples = TRUE;
    pmt->bTemporalCompression = FALSE;

    bool isNv12 = (subtype == MEDIASUBTYPE_NV12);
    bool isRgb32 = (subtype == MEDIASUBTYPE_RGB32);
    size_t sampleSize = isNv12 ? (width * height * 3 / 2) : (isRgb32 ? (width * height * 4) : (width * height * 3));
    pmt->lSampleSize = (ULONG)sampleSize;
    pmt->formattype = FORMAT_VideoInfo;
    pmt->cbFormat = sizeof(VIDEOINFOHEADER);
    pmt->pbFormat = (BYTE*)CoTaskMemAlloc(sizeof(VIDEOINFOHEADER));
    ZeroMemory(pmt->pbFormat, sizeof(VIDEOINFOHEADER));

    VIDEOINFOHEADER* vih = (VIDEOINFOHEADER*)pmt->pbFormat;
    vih->rcSource.left = 0;
    vih->rcSource.top = 0;
    vih->rcSource.right = width;
    vih->rcSource.bottom = height;
    vih->rcTarget = vih->rcSource;
    vih->AvgTimePerFrame = (REFERENCE_TIME)(10000000 / fps);
    vih->dwBitRate = (DWORD)(sampleSize * 8 * fps);

    vih->bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    vih->bmiHeader.biWidth = width;
    vih->bmiHeader.biHeight = isNv12 ? height : height;
    vih->bmiHeader.biPlanes = 1;
    vih->bmiHeader.biBitCount = isNv12 ? 12 : (isRgb32 ? 32 : 24);
    vih->bmiHeader.biCompression = isNv12 ? MAKEFOURCC('N','V','1','2') : BI_RGB;
    vih->bmiHeader.biSizeImage = (DWORD)sampleSize;
}

static HRESULT CopyMediaType(AM_MEDIA_TYPE* pDest, const AM_MEDIA_TYPE* pSrc) {
    if (!pDest || !pSrc) return E_POINTER;
    *pDest = *pSrc;
    if (pSrc->cbFormat > 0 && pSrc->pbFormat) {
        pDest->pbFormat = (BYTE*)CoTaskMemAlloc(pSrc->cbFormat);
        if (!pDest->pbFormat) return E_OUTOFMEMORY;
        CopyMemory(pDest->pbFormat, pSrc->pbFormat, pSrc->cbFormat);
    } else {
        pDest->cbFormat = 0;
        pDest->pbFormat = NULL;
    }
    if (pDest->pUnk) {
        pDest->pUnk->AddRef();
    }
    return S_OK;
}

static void FreeMediaType(AM_MEDIA_TYPE& mt) {
    if (mt.cbFormat != 0 && mt.pbFormat) {
        CoTaskMemFree(mt.pbFormat);
        mt.cbFormat = 0;
        mt.pbFormat = NULL;
    }
    if (mt.pUnk) {
        mt.pUnk->Release();
        mt.pUnk = NULL;
    }
}

// ----------------------------------------------------------------------------
// DirectShow Output Pin with IAMStreamConfig & IEnumMediaTypes
// ----------------------------------------------------------------------------
class LinkCamOutputPin : public IPin, public IKsPropertySet, public IAMStreamConfig {
public:
    LinkCamFilter* m_pFilter;
    IPin* m_pConnectedPin = NULL;
    IMemInputPin* m_pInputPin = NULL;
    IMemAllocator* m_pAllocator = NULL;
    std::atomic<long> m_refCount{1};
    ShmReader m_shm;
    HANDLE m_hThread = NULL;
    std::atomic<bool> m_bRunning{false};
    uint32_t m_lastSequence = 0;
    AM_MEDIA_TYPE m_currentMediaType;

    LinkCamOutputPin(LinkCamFilter* pFilter);
    ~LinkCamOutputPin();

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override;
    STDMETHODIMP_(ULONG) AddRef() override { return ++m_refCount; }
    STDMETHODIMP_(ULONG) Release() override {
        ULONG c = --m_refCount;
        if (c == 0) delete this;
        return c;
    }

    // IPin
    STDMETHODIMP Connect(IPin* pReceivePin, const AM_MEDIA_TYPE* pmt) override;
    STDMETHODIMP ReceiveConnection(IPin* pConnector, const AM_MEDIA_TYPE* pmt) override { return E_UNEXPECTED; }
    STDMETHODIMP Disconnect() override;
    STDMETHODIMP ConnectedTo(IPin** ppPin) override;
    STDMETHODIMP ConnectionMediaType(AM_MEDIA_TYPE* pmt) override;
    STDMETHODIMP QueryPinInfo(PIN_INFO* pInfo) override;
    STDMETHODIMP QueryDirection(PIN_DIRECTION* pPinDir) override { *pPinDir = PINDIR_OUTPUT; return S_OK; }
    STDMETHODIMP QueryId(LPWSTR* Id) override;
    STDMETHODIMP QueryAccept(const AM_MEDIA_TYPE* pmt) override;
    STDMETHODIMP EnumMediaTypes(IEnumMediaTypes** ppEnum) override;
    STDMETHODIMP QueryInternalConnections(IPin** apPin, ULONG* nPin) override { return E_NOTIMPL; }
    STDMETHODIMP EndOfStream() override { return S_OK; }
    STDMETHODIMP BeginFlush() override { return S_OK; }
    STDMETHODIMP EndFlush() override { return S_OK; }
    STDMETHODIMP NewSegment(REFERENCE_TIME tStart, REFERENCE_TIME tStop, double dRate) override { return S_OK; }

    // IKsPropertySet
    STDMETHODIMP Set(REFGUID guidPropSet, DWORD dwPropID, LPVOID pInstanceData, DWORD cbInstanceData, LPVOID pPropData, DWORD cbPropData) override { return E_NOTIMPL; }
    STDMETHODIMP Get(REFGUID guidPropSet, DWORD dwPropID, LPVOID pInstanceData, DWORD cbInstanceData, LPVOID pPropData, DWORD cbPropData, DWORD* pcbReturned) override;
    STDMETHODIMP QuerySupported(REFGUID guidPropSet, DWORD dwPropID, DWORD* pTypeSupport) override;

    // IAMStreamConfig
    STDMETHODIMP SetFormat(AM_MEDIA_TYPE* pmt) override;
    STDMETHODIMP GetFormat(AM_MEDIA_TYPE** ppmt) override;
    STDMETHODIMP GetNumberOfCapabilities(int* piCount, int* piSize) override;
    STDMETHODIMP GetStreamCaps(int iIndex, AM_MEDIA_TYPE** ppmt, BYTE* pSCC) override;

    void StartStreaming();
    void StopStreaming();
    static DWORD WINAPI ThreadProc(LPVOID lpParam);
};

// ----------------------------------------------------------------------------
// Single Pin Enumerator
// ----------------------------------------------------------------------------
class SinglePinEnum : public IEnumPins {
    std::atomic<long> m_ref{1};
    IPin* m_pPin;
    ULONG m_pos = 0;
public:
    SinglePinEnum(IPin* pPin) : m_pPin(pPin) { if (m_pPin) m_pPin->AddRef(); }
    ~SinglePinEnum() { if (m_pPin) m_pPin->Release(); }

    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_POINTER;
        if (riid == IID_IUnknown || riid == __uuidof(IEnumPins)) { *ppv = this; AddRef(); return S_OK; }
        *ppv = NULL;
        return E_NOINTERFACE;
    }
    STDMETHODIMP_(ULONG) AddRef() override { return ++m_ref; }
    STDMETHODIMP_(ULONG) Release() override {
        ULONG c = --m_ref;
        if (c == 0) delete this;
        return c;
    }
    STDMETHODIMP Next(ULONG cPins, IPin** ppPins, ULONG* pcFetched) override {
        if (!ppPins) return E_POINTER;
        if (m_pos == 0 && cPins > 0 && m_pPin) {
            ppPins[0] = m_pPin;
            m_pPin->AddRef();
            m_pos = 1;
            if (pcFetched) *pcFetched = 1;
            return (cPins == 1) ? S_OK : S_FALSE;
        }
        if (pcFetched) *pcFetched = 0;
        return S_FALSE;
    }
    STDMETHODIMP Skip(ULONG cPins) override { m_pos += cPins; return S_OK; }
    STDMETHODIMP Reset() override { m_pos = 0; return S_OK; }
    STDMETHODIMP Clone(IEnumPins** ppEnum) override { return E_NOTIMPL; }
};

// ----------------------------------------------------------------------------
// Media Types Enumerator
// Proposing standard formats: NV12 1080p60, NV12 1080p30, RGB24 1080p60, RGB24 1080p30
// ----------------------------------------------------------------------------
class EnumMediaTypesImpl : public IEnumMediaTypes {
    std::atomic<long> m_ref{1};
    ULONG m_pos = 0;
    LinkCamOutputPin* m_pPin = NULL;
public:
    EnumMediaTypesImpl(LinkCamOutputPin* pPin) : m_pPin(pPin) {
        if (m_pPin) m_pPin->AddRef();
    }
    virtual ~EnumMediaTypesImpl() {
        if (m_pPin) m_pPin->Release();
    }

    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_POINTER;
        if (riid == IID_IUnknown || riid == __uuidof(IEnumMediaTypes)) {
            *ppv = this;
            AddRef();
            return S_OK;
        }
        *ppv = NULL;
        return E_NOINTERFACE;
    }
    STDMETHODIMP_(ULONG) AddRef() override { return ++m_ref; }
    STDMETHODIMP_(ULONG) Release() override {
        ULONG c = --m_ref;
        if (c == 0) delete this;
        return c;
    }

    STDMETHODIMP Next(ULONG cMediaTypes, AM_MEDIA_TYPE** ppMediaTypes, ULONG* pcFetched) override {
        if (!ppMediaTypes) return E_POINTER;
        ULONG fetched = 0;
        while (m_pos < 4 && fetched < cMediaTypes) {
            ppMediaTypes[fetched] = (AM_MEDIA_TYPE*)CoTaskMemAlloc(sizeof(AM_MEDIA_TYPE));
            if (!ppMediaTypes[fetched]) break;

            bool isNv12 = (m_pos < 2);
            int fps = (m_pos % 2 == 0) ? 60 : 30;
            BuildMediaType(ppMediaTypes[fetched], 1920, 1080, fps,
                           isNv12 ? MEDIASUBTYPE_NV12 : MEDIASUBTYPE_RGB24);
            fetched++;
            m_pos++;
        }
        if (pcFetched) *pcFetched = fetched;
        return (fetched == cMediaTypes) ? S_OK : S_FALSE;
    }

    STDMETHODIMP Skip(ULONG cMediaTypes) override {
        m_pos += cMediaTypes;
        return (m_pos <= 4) ? S_OK : S_FALSE;
    }

    STDMETHODIMP Reset() override {
        m_pos = 0;
        return S_OK;
    }

    STDMETHODIMP Clone(IEnumMediaTypes** ppEnum) override {
        if (!ppEnum) return E_POINTER;
        EnumMediaTypesImpl* pNew = new EnumMediaTypesImpl(m_pPin);
        pNew->m_pos = m_pos;
        *ppEnum = pNew;
        return S_OK;
    }
};

// ----------------------------------------------------------------------------
// Minimal DirectShow Filter
// ----------------------------------------------------------------------------
class LinkCamFilter : public IBaseFilter {
public:
    std::atomic<long> m_refCount{1};
    FILTER_STATE m_state = State_Stopped;
    LinkCamOutputPin* m_pPin = NULL;
    IFilterGraph* m_pGraph = NULL;

    LinkCamFilter() {
        m_pPin = new LinkCamOutputPin(this);
        ++g_cLock;
    }
    virtual ~LinkCamFilter() {
        if (m_pPin) { m_pPin->Release(); m_pPin = NULL; }
        --g_cLock;
    }

    // IUnknown
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_POINTER;
        if (riid == IID_IUnknown || riid == IID_IPersist || riid == __uuidof(IMediaFilter) || riid == __uuidof(IBaseFilter)) {
            *ppv = static_cast<IBaseFilter*>(this);
            AddRef();
            return S_OK;
        }
        *ppv = NULL;
        return E_NOINTERFACE;
    }
    STDMETHODIMP_(ULONG) AddRef() override { return ++m_refCount; }
    STDMETHODIMP_(ULONG) Release() override {
        ULONG c = --m_refCount;
        if (c == 0) delete this;
        return c;
    }

    // IPersist
    STDMETHODIMP GetClassID(CLSID* pClsid) override {
        if (!pClsid) return E_POINTER;
        *pClsid = CLSID_LinkCamDShowFilter;
        return S_OK;
    }

    // IMediaFilter
    STDMETHODIMP Stop() override {
        m_state = State_Stopped;
        if (m_pPin) m_pPin->StopStreaming();
        return S_OK;
    }
    STDMETHODIMP Pause() override {
        m_state = State_Paused;
        return S_OK;
    }
    STDMETHODIMP Run(REFERENCE_TIME tStart) override {
        m_state = State_Running;
        if (m_pPin) m_pPin->StartStreaming();
        return S_OK;
    }
    STDMETHODIMP GetState(DWORD dwMilliSecsTimeout, FILTER_STATE* State) override {
        if (!State) return E_POINTER;
        *State = m_state;
        return S_OK;
    }
    STDMETHODIMP SetSyncSource(IReferenceClock* pClock) override { return S_OK; }
    STDMETHODIMP GetSyncSource(IReferenceClock** pClock) override { if (pClock) *pClock = NULL; return S_OK; }

    // IBaseFilter
    STDMETHODIMP EnumPins(IEnumPins** ppEnum) override {
        if (!ppEnum) return E_POINTER;
        *ppEnum = new SinglePinEnum(m_pPin);
        return S_OK;
    }
    STDMETHODIMP FindPin(LPCWSTR Id, IPin** ppPin) override {
        if (!ppPin) return E_POINTER;
        if (m_pPin) { *ppPin = m_pPin; m_pPin->AddRef(); return S_OK; }
        return VFW_E_NOT_FOUND;
    }
    STDMETHODIMP QueryFilterInfo(FILTER_INFO* pInfo) override {
        if (!pInfo) return E_POINTER;
        wcscpy_s(pInfo->achName, L"LinkCam Virtual Camera");
        pInfo->pGraph = m_pGraph;
        if (m_pGraph) m_pGraph->AddRef();
        return S_OK;
    }
    STDMETHODIMP JoinFilterGraph(IFilterGraph* pGraph, LPCWSTR pName) override {
        m_pGraph = pGraph;
        return S_OK;
    }
    STDMETHODIMP QueryVendorInfo(LPWSTR* pVendorInfo) override { return E_NOTIMPL; }
};

// ----------------------------------------------------------------------------
// Pin Implementation Details
// ----------------------------------------------------------------------------
LinkCamOutputPin::LinkCamOutputPin(LinkCamFilter* pFilter) : m_pFilter(pFilter) {
    BuildMediaType(&m_currentMediaType, 1920, 1080, 60, MEDIASUBTYPE_NV12);
}

LinkCamOutputPin::~LinkCamOutputPin() {
    StopStreaming();
    Disconnect();
    FreeMediaType(m_currentMediaType);
}

STDMETHODIMP LinkCamOutputPin::QueryInterface(REFIID riid, void** ppv) {
    if (!ppv) return E_POINTER;
    if (riid == IID_IUnknown || riid == __uuidof(IPin)) {
        *ppv = static_cast<IPin*>(this);
        AddRef();
        return S_OK;
    }
    if (riid == IID_IKsPropertySet || riid == __uuidof(IKsPropertySet)) {
        *ppv = static_cast<IKsPropertySet*>(this);
        AddRef();
        return S_OK;
    }
    if (riid == IID_IAMStreamConfig || riid == __uuidof(IAMStreamConfig)) {
        *ppv = static_cast<IAMStreamConfig*>(this);
        AddRef();
        return S_OK;
    }
    *ppv = NULL;
    return E_NOINTERFACE;
}

STDMETHODIMP LinkCamOutputPin::Connect(IPin* pReceivePin, const AM_MEDIA_TYPE* pmt) {
    if (!pReceivePin) return E_POINTER;
    if (m_pConnectedPin) return VFW_E_ALREADY_CONNECTED;

    AM_MEDIA_TYPE mt;
    ZeroMemory(&mt, sizeof(AM_MEDIA_TYPE));

    if (pmt) {
        if (QueryAccept(pmt) != S_OK) return VFW_E_TYPE_NOT_ACCEPTED;
        CopyMediaType(&mt, pmt);
    } else {
        BuildMediaType(&mt, 1920, 1080, 60, MEDIASUBTYPE_NV12);
    }

    HRESULT hr = pReceivePin->ReceiveConnection(this, &mt);
    if (FAILED(hr)) {
        FreeMediaType(mt);
        BuildMediaType(&mt, 1920, 1080, 60, MEDIASUBTYPE_RGB24);
        hr = pReceivePin->ReceiveConnection(this, &mt);
        if (FAILED(hr)) {
            FreeMediaType(mt);
            return hr;
        }
    }

    m_pConnectedPin = pReceivePin;
    m_pConnectedPin->AddRef();
    FreeMediaType(m_currentMediaType);
    CopyMediaType(&m_currentMediaType, &mt);
    FreeMediaType(mt);

    pReceivePin->QueryInterface(__uuidof(IMemInputPin), (void**)&m_pInputPin);
    if (m_pInputPin) {
        hr = m_pInputPin->GetAllocator(&m_pAllocator);
        if (FAILED(hr) || !m_pAllocator) {
            hr = CoCreateInstance(CLSID_MemoryAllocator, NULL, CLSCTX_INPROC_SERVER,
                                  IID_IMemAllocator, (void**)&m_pAllocator);
        }
        if (m_pAllocator) {
            ALLOCATOR_PROPERTIES prop = {0}, actual = {0};
            prop.cBuffers = 3;
            prop.cbBuffer = (long)std::max<size_t>(m_currentMediaType.lSampleSize, 1920 * 1080 * 4);
            prop.cbAlign = 1;
            m_pAllocator->SetProperties(&prop, &actual);
            m_pInputPin->NotifyAllocator(m_pAllocator, FALSE);
            m_pAllocator->Commit();
        }
    }

    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::Disconnect() {
    if (!m_pConnectedPin) return S_FALSE;
    if (m_pAllocator) {
        m_pAllocator->Decommit();
        m_pAllocator->Release();
        m_pAllocator = NULL;
    }
    if (m_pInputPin) {
        m_pInputPin->Release();
        m_pInputPin = NULL;
    }
    if (m_pConnectedPin) {
        m_pConnectedPin->Release();
        m_pConnectedPin = NULL;
    }
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::ConnectedTo(IPin** ppPin) {
    if (!ppPin) return E_POINTER;
    if (!m_pConnectedPin) return VFW_E_NOT_CONNECTED;
    *ppPin = m_pConnectedPin;
    m_pConnectedPin->AddRef();
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::ConnectionMediaType(AM_MEDIA_TYPE* pmt) {
    if (!pmt) return E_POINTER;
    if (!m_pConnectedPin) return VFW_E_NOT_CONNECTED;
    return CopyMediaType(pmt, &m_currentMediaType);
}

STDMETHODIMP LinkCamOutputPin::QueryPinInfo(PIN_INFO* pInfo) {
    if (!pInfo) return E_POINTER;
    pInfo->pFilter = m_pFilter;
    if (m_pFilter) m_pFilter->AddRef();
    pInfo->dir = PINDIR_OUTPUT;
    wcscpy_s(pInfo->achName, L"Capture");
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::QueryId(LPWSTR* Id) {
    if (!Id) return E_POINTER;
    *Id = (LPWSTR)CoTaskMemAlloc(sizeof(L"Capture"));
    if (*Id) wcscpy_s(*Id, 8, L"Capture");
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::QueryAccept(const AM_MEDIA_TYPE* pmt) {
    if (!pmt) return E_POINTER;
    if (pmt->majortype == MEDIATYPE_Video) {
        if (pmt->subtype == MEDIASUBTYPE_NV12 ||
            pmt->subtype == MEDIASUBTYPE_RGB24 ||
            pmt->subtype == MEDIASUBTYPE_RGB32) {
            return S_OK;
        }
    }
    return S_FALSE;
}

STDMETHODIMP LinkCamOutputPin::EnumMediaTypes(IEnumMediaTypes** ppEnum) {
    if (!ppEnum) return E_POINTER;
    *ppEnum = new EnumMediaTypesImpl(this);
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::Get(REFGUID guidPropSet, DWORD dwPropID, LPVOID pInstanceData, DWORD cbInstanceData, LPVOID pPropData, DWORD cbPropData, DWORD* pcbReturned) {
    if (guidPropSet == LinkCam_AMPROPSETID_Pin && dwPropID == AMPROPERTY_PIN_CATEGORY) {
        if (pcbReturned) *pcbReturned = sizeof(GUID);
        if (pPropData && cbPropData >= sizeof(GUID)) {
            *(GUID*)pPropData = LinkCam_PIN_CATEGORY_CAPTURE;
            return S_OK;
        }
        return E_FAIL;
    }
    return E_NOTIMPL;
}

STDMETHODIMP LinkCamOutputPin::QuerySupported(REFGUID guidPropSet, DWORD dwPropID, DWORD* pTypeSupport) {
    if (guidPropSet == LinkCam_AMPROPSETID_Pin && dwPropID == AMPROPERTY_PIN_CATEGORY) {
        if (pTypeSupport) *pTypeSupport = KSPROPERTY_SUPPORT_GET;
        return S_OK;
    }
    return E_NOTIMPL;
}

// ----------------------------------------------------------------------------
// IAMStreamConfig Implementation
// ----------------------------------------------------------------------------
STDMETHODIMP LinkCamOutputPin::SetFormat(AM_MEDIA_TYPE* pmt) {
    if (!pmt) return E_POINTER;
    if (QueryAccept(pmt) != S_OK) return VFW_E_TYPE_NOT_ACCEPTED;
    FreeMediaType(m_currentMediaType);
    CopyMediaType(&m_currentMediaType, pmt);
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::GetFormat(AM_MEDIA_TYPE** ppmt) {
    if (!ppmt) return E_POINTER;
    *ppmt = (AM_MEDIA_TYPE*)CoTaskMemAlloc(sizeof(AM_MEDIA_TYPE));
    if (!*ppmt) return E_OUTOFMEMORY;
    CopyMediaType(*ppmt, &m_currentMediaType);
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::GetNumberOfCapabilities(int* piCount, int* piSize) {
    if (!piCount || !piSize) return E_POINTER;
    *piCount = 4;
    *piSize = sizeof(VIDEO_STREAM_CONFIG_CAPS);
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::GetStreamCaps(int iIndex, AM_MEDIA_TYPE** ppmt, BYTE* pSCC) {
    if (!ppmt || !pSCC) return E_POINTER;
    if (iIndex < 0 || iIndex >= 4) return S_FALSE;

    *ppmt = (AM_MEDIA_TYPE*)CoTaskMemAlloc(sizeof(AM_MEDIA_TYPE));
    if (!*ppmt) return E_OUTOFMEMORY;

    bool isNv12 = (iIndex < 2);
    int fps = (iIndex % 2 == 0) ? 60 : 30;
    BuildMediaType(*ppmt, 1920, 1080, fps, isNv12 ? MEDIASUBTYPE_NV12 : MEDIASUBTYPE_RGB24);

    VIDEO_STREAM_CONFIG_CAPS* caps = (VIDEO_STREAM_CONFIG_CAPS*)pSCC;
    ZeroMemory(caps, sizeof(VIDEO_STREAM_CONFIG_CAPS));
    caps->guid = MEDIATYPE_Video;
    caps->VideoStandard = 0;
    caps->InputSize.cx = 1920;
    caps->InputSize.cy = 1080;
    caps->MinCroppingSize.cx = 1920;
    caps->MinCroppingSize.cy = 1080;
    caps->MaxCroppingSize.cx = 1920;
    caps->MaxCroppingSize.cy = 1080;
    caps->CropGranularityX = 1;
    caps->CropGranularityY = 1;
    caps->MinOutputSize.cx = 640;
    caps->MinOutputSize.cy = 360;
    caps->MaxOutputSize.cx = 1920;
    caps->MaxOutputSize.cy = 1080;
    caps->OutputGranularityX = 16;
    caps->OutputGranularityY = 16;
    caps->MinFrameInterval = 10000000 / 60;
    caps->MaxFrameInterval = 10000000 / 30;
    caps->MinBitsPerSecond = 5000000;
    caps->MaxBitsPerSecond = 50000000;

    return S_OK;
}

void LinkCamOutputPin::StartStreaming() {
    if (m_bRunning.exchange(true)) return;
    m_shm.Open();
    m_hThread = CreateThread(NULL, 0, ThreadProc, this, 0, NULL);
}

void LinkCamOutputPin::StopStreaming() {
    m_bRunning = false;
    if (m_hThread) {
        WaitForSingleObject(m_hThread, 500);
        CloseHandle(m_hThread);
        m_hThread = NULL;
    }
    m_shm.Close();
}

// ----------------------------------------------------------------------------
// Streaming Thread Procedure
// ----------------------------------------------------------------------------
DWORD WINAPI LinkCamOutputPin::ThreadProc(LPVOID lpParam) {
    LinkCamOutputPin* pin = (LinkCamOutputPin*)lpParam;
    REFERENCE_TIME rtTime = 0;
    const REFERENCE_TIME rtFrameDuration = 10000000 / 60; // 60 FPS (16.6ms)

    MftH264Decoder decoder;
    std::vector<uint8_t> decodedNv12;
    std::vector<uint8_t> rgbBuffer;
    uint32_t frameCount = 0;

    while (pin->m_bRunning) {
        frameCount++;
        bool hasNewFrame = pin->m_shm.WaitForFrame(17);

        if (!pin->m_pInputPin || !pin->m_pAllocator) {
            Sleep(10);
            continue;
        }

        IMediaSample* pSample = NULL;
        if (FAILED(pin->m_pAllocator->GetBuffer(&pSample, NULL, NULL, 0)) || !pSample) {
            Sleep(5);
            continue;
        }

        BYTE* pData = NULL;
        if (FAILED(pSample->GetPointer(&pData)) || !pData) {
            pSample->Release();
            continue;
        }

        long sampleCapacity = pSample->GetSize();
        GUID subtype = pin->m_currentMediaType.subtype;
        bool isNv12 = (subtype == MEDIASUBTYPE_NV12);
        bool isRgb32 = (subtype == MEDIASUBTYPE_RGB32);
        long targetSize = isNv12 ? (1920 * 1080 * 3 / 2) : (isRgb32 ? (1920 * 1080 * 4) : (1920 * 1080 * 3));

        const auto* hdr = pin->m_shm.GetHeader();
        const uint8_t* payload = pin->m_shm.GetPayload();
        const uint32_t maxAllowedPayload = (32 * 1024 * 1024) - sizeof(ShmVideoHeader);

        bool frameValid = false;
        if (hasNewFrame && hdr && hdr->magic == SHM_MAGIC && hdr->version == 1 &&
            hdr->payload_size > 0 && hdr->payload_size <= maxAllowedPayload && payload) {

            uint32_t startSeq = hdr->sequence;
            if ((startSeq & 1) == 0 && startSeq != pin->m_lastSequence) {
                if (hdr->format == 0) { // H.264
                    if (decoder.Decode(payload, hdr->payload_size, hdr->timestamp_us, decodedNv12)) {
                        if (hdr->sequence == startSeq) {
                            pin->m_lastSequence = startSeq;
                            frameValid = true;

                            if (isNv12) {
                                long copyLen = (long)(std::min)((size_t)sampleCapacity, decodedNv12.size());
                                memcpy(pData, decodedNv12.data(), copyLen);
                                pSample->SetActualDataLength(copyLen);
                            } else if (isRgb32) {
                                if (rgbBuffer.size() != 1920 * 1080 * 4) rgbBuffer.resize(1920 * 1080 * 4);
                                Nv12ToRgb32(decodedNv12.data(), rgbBuffer.data(), 1920, 1080);
                                long copyLen = (long)(std::min)((size_t)sampleCapacity, rgbBuffer.size());
                                memcpy(pData, rgbBuffer.data(), copyLen);
                                pSample->SetActualDataLength(copyLen);
                            } else {
                                if (rgbBuffer.size() != 1920 * 1080 * 3) rgbBuffer.resize(1920 * 1080 * 3);
                                Nv12ToRgb24(decodedNv12.data(), rgbBuffer.data(), 1920, 1080);
                                long copyLen = (long)(std::min)((size_t)sampleCapacity, rgbBuffer.size());
                                memcpy(pData, rgbBuffer.data(), copyLen);
                                pSample->SetActualDataLength(copyLen);
                            }
                        }
                    }
                } else if (hdr->format == 1) { // NV12 directly
                    if (hdr->sequence == startSeq) {
                        pin->m_lastSequence = startSeq;
                        frameValid = true;
                        if (isNv12) {
                            long copyLen = (long)(std::min)((uint32_t)sampleCapacity, hdr->payload_size);
                            memcpy(pData, payload, copyLen);
                            pSample->SetActualDataLength(copyLen);
                        } else {
                            if (rgbBuffer.size() != 1920 * 1080 * 3) rgbBuffer.resize(1920 * 1080 * 3);
                            Nv12ToRgb24(payload, rgbBuffer.data(), 1920, 1080);
                            long copyLen = (long)(std::min)((size_t)sampleCapacity, rgbBuffer.size());
                            memcpy(pData, rgbBuffer.data(), copyLen);
                            pSample->SetActualDataLength(copyLen);
                        }
                    }
                }
            }
        }

        // Deliver standby frame if waiting for phone camera stream
        if (!frameValid) {
            long len = (long)(std::min)((long)sampleCapacity, targetSize);
            RenderStandbyFrame(pData, 1920, 1080, subtype, frameCount);
            pSample->SetActualDataLength(len);
        }

        REFERENCE_TIME rtStart = rtTime;
        REFERENCE_TIME rtEnd = rtTime + rtFrameDuration;
        pSample->SetTime(&rtStart, &rtEnd);
        pSample->SetSyncPoint(TRUE);

        pin->m_pInputPin->Receive(pSample);
        pSample->Release();
        rtTime += rtFrameDuration;
    }
    return 0;
}

// ----------------------------------------------------------------------------
// COM ClassFactory
// ----------------------------------------------------------------------------
class LinkCamClassFactory : public IClassFactory {
    std::atomic<long> m_ref{1};
public:
    STDMETHODIMP QueryInterface(REFIID riid, void** ppv) override {
        if (!ppv) return E_POINTER;
        if (riid == IID_IUnknown || riid == IID_IClassFactory) { *ppv = this; AddRef(); return S_OK; }
        *ppv = NULL;
        return E_NOINTERFACE;
    }
    STDMETHODIMP_(ULONG) AddRef() override { return ++m_ref; }
    STDMETHODIMP_(ULONG) Release() override {
        ULONG c = --m_ref;
        if (c == 0) delete this;
        return c;
    }
    STDMETHODIMP CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppv) override {
        if (pUnkOuter) return CLASS_E_NOAGGREGATION;
        LinkCamFilter* pFilter = new LinkCamFilter();
        HRESULT hr = pFilter->QueryInterface(riid, ppv);
        pFilter->Release();
        return hr;
    }
    STDMETHODIMP LockServer(BOOL fLock) override {
        if (fLock) ++g_cLock; else --g_cLock;
        return S_OK;
    }
};

// ----------------------------------------------------------------------------
// DLL Entry Points
// ----------------------------------------------------------------------------
BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID lpReserved) {
    if (ul_reason_for_call == DLL_PROCESS_ATTACH) {
        g_hInst = hModule;
        DisableThreadLibraryCalls(hModule);
    }
    return TRUE;
}

STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, void** ppv) {
    if (rclsid == CLSID_LinkCamDShowFilter) {
        LinkCamClassFactory* pFactory = new LinkCamClassFactory();
        HRESULT hr = pFactory->QueryInterface(riid, ppv);
        pFactory->Release();
        return hr;
    }
    return CLASS_E_CLASSNOTAVAILABLE;
}

STDAPI DllCanUnloadNow() {
    return (g_cLock == 0) ? S_OK : S_FALSE;
}

STDAPI DllRegisterServer() {
    WCHAR szModule[MAX_PATH];
    if (!GetModuleFileNameW(g_hInst, szModule, MAX_PATH)) return E_FAIL;

    // 1. Register CLSID\{D37E5526-92F7-4E61-A76F-B590E8F88E29}
    const WCHAR* clsidKey = L"Software\\Classes\\CLSID\\{D37E5526-92F7-4E61-A76F-B590E8F88E29}";
    HKEY hKey = NULL;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, clsidKey, 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS) {
        const WCHAR* friendlyName = L"LinkCam Virtual Camera";
        RegSetValueExW(hKey, NULL, 0, REG_SZ, (const BYTE*)friendlyName, (DWORD)((wcslen(friendlyName) + 1) * sizeof(WCHAR)));

        HKEY hSubKey = NULL;
        if (RegCreateKeyExW(hKey, L"InprocServer32", 0, NULL, 0, KEY_WRITE, NULL, &hSubKey, NULL) == ERROR_SUCCESS) {
            RegSetValueExW(hSubKey, NULL, 0, REG_SZ, (const BYTE*)szModule, (DWORD)((wcslen(szModule) + 1) * sizeof(WCHAR)));
            const WCHAR* threading = L"Both";
            RegSetValueExW(hSubKey, L"ThreadingModel", 0, REG_SZ, (const BYTE*)threading, (DWORD)((wcslen(threading) + 1) * sizeof(WCHAR)));
            RegCloseKey(hSubKey);
        }
        RegCloseKey(hKey);
    }

    // 2. Register under DirectShow Video Input Category {860BB310-5D01-11D0-BD3B-00A0C911CE86}
    const WCHAR* catKey = L"Software\\Classes\\CLSID\\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\\Instance\\{D37E5526-92F7-4E61-A76F-B590E8F88E29}";
    if (RegCreateKeyExW(HKEY_CURRENT_USER, catKey, 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS) {
        const WCHAR* name = L"LinkCam Virtual Camera";
        RegSetValueExW(hKey, L"FriendlyName", 0, REG_SZ, (const BYTE*)name, (DWORD)((wcslen(name) + 1) * sizeof(WCHAR)));
        const WCHAR* filterClsid = L"{D37E5526-92F7-4E61-A76F-B590E8F88E29}";
        RegSetValueExW(hKey, L"CLSID", 0, REG_SZ, (const BYTE*)filterClsid, (DWORD)((wcslen(filterClsid) + 1) * sizeof(WCHAR)));
        RegCloseKey(hKey);
    }
    return S_OK;
}

STDAPI DllUnregisterServer() {
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\Classes\\CLSID\\{D37E5526-92F7-4E61-A76F-B590E8F88E29}\\InprocServer32");
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\Classes\\CLSID\\{D37E5526-92F7-4E61-A76F-B590E8F88E29}");
    RegDeleteKeyW(HKEY_CURRENT_USER, L"Software\\Classes\\CLSID\\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\\Instance\\{D37E5526-92F7-4E61-A76F-B590E8F88E29}");
    return S_OK;
}
