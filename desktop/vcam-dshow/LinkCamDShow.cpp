// LinkCam DirectShow Virtual Camera Filter DLL
// Lightweight, native Windows COM InprocServer32 bridge reading from Local\LinkCam_Video_Shm

#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <initguid.h>
#include <dshow.h>
#include <cstdint>
#include <atomic>
#include <algorithm>

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
// Minimal DirectShow Output Pin
// ----------------------------------------------------------------------------
class LinkCamOutputPin : public IPin, public IKsPropertySet {
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

    void StartStreaming();
    void StopStreaming();
    static DWORD WINAPI ThreadProc(LPVOID lpParam);
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
    STDMETHODIMP EnumPins(IEnumPins** ppEnum) override;
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
LinkCamOutputPin::LinkCamOutputPin(LinkCamFilter* pFilter) : m_pFilter(pFilter) {}
LinkCamOutputPin::~LinkCamOutputPin() {
    StopStreaming();
    Disconnect();
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
    *ppv = NULL;
    return E_NOINTERFACE;
}

STDMETHODIMP LinkCamOutputPin::Connect(IPin* pReceivePin, const AM_MEDIA_TYPE* pmt) {
    if (!pReceivePin) return E_POINTER;
    if (m_pConnectedPin) return VFW_E_ALREADY_CONNECTED;

    m_pConnectedPin = pReceivePin;
    m_pConnectedPin->AddRef();

    pReceivePin->QueryInterface(__uuidof(IMemInputPin), (void**)&m_pInputPin);
    if (m_pInputPin) {
        m_pInputPin->GetAllocator(&m_pAllocator);
        if (m_pAllocator) {
            ALLOCATOR_PROPERTIES prop = {0}, actual = {0};
            prop.cBuffers = 2;
            prop.cbBuffer = 3840 * 2160 * 4; // 4K uncompressed max capacity (supports 1080p and 4K)
            prop.cbAlign = 1;
            m_pAllocator->SetProperties(&prop, &actual);
            m_pAllocator->Commit();
        }
    }
    return S_OK;
}

STDMETHODIMP LinkCamOutputPin::Disconnect() {
    if (m_pAllocator) { m_pAllocator->Decommit(); m_pAllocator->Release(); m_pAllocator = NULL; }
    if (m_pInputPin) { m_pInputPin->Release(); m_pInputPin = NULL; }
    if (m_pConnectedPin) { m_pConnectedPin->Release(); m_pConnectedPin = NULL; }
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
    // Deliver RGB32 / NV12 format description
    ZeroMemory(pmt, sizeof(AM_MEDIA_TYPE));
    pmt->majortype = MEDIATYPE_Video;
    pmt->subtype = MEDIASUBTYPE_RGB24;
    pmt->formattype = FORMAT_VideoInfo;
    return S_OK;
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
    if (pmt->majortype == MEDIATYPE_Video) return S_OK;
    return S_FALSE;
}

STDMETHODIMP LinkCamOutputPin::EnumMediaTypes(IEnumMediaTypes** ppEnum) {
    if (!ppEnum) return E_POINTER;
    *ppEnum = NULL;
    return E_NOTIMPL; // Downstream renderer will propose formats
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

DWORD WINAPI LinkCamOutputPin::ThreadProc(LPVOID lpParam) {
    LinkCamOutputPin* pin = (LinkCamOutputPin*)lpParam;
    REFERENCE_TIME rtTime = 0;
    const REFERENCE_TIME rtFrameDuration = 10000000 / 60; // 60 FPS (16.6ms)

    while (pin->m_bRunning) {
        // Wait for next frame from Rust Shared Memory (or 16ms timeout fallback)
        pin->m_shm.WaitForFrame(17);

        if (!pin->m_pInputPin || !pin->m_pAllocator) {
            Sleep(10);
            continue;
        }

        // Strict metadata validation at IPC trust boundary
        const auto* hdr = pin->m_shm.GetHeader();
        const uint8_t* payload = pin->m_shm.GetPayload();

        const uint32_t maxAllowedPayload = (32 * 1024 * 1024) - sizeof(ShmVideoHeader);
        if (!hdr || hdr->magic != SHM_MAGIC || hdr->version != 1 ||
            hdr->width < 16 || hdr->width > 4096 ||
            hdr->height < 16 || hdr->height > 2160 ||
            hdr->payload_size == 0 || hdr->payload_size > maxAllowedPayload ||
            !payload) {
            Sleep(5);
            continue;
        }

        // Seqlock verification: if odd, write is currently in progress; wait and retry
        uint32_t startSeq = hdr->sequence;
        if ((startSeq & 1) != 0) {
            Sleep(1);
            continue;
        }

        // Multi-Reader Deduplication: Check sequence to prevent re-processing identical frames
        // while allowing concurrent readers (e.g. Zoom and OBS) to both process the frame.
        if (startSeq == pin->m_lastSequence) {
            Sleep(1);
            continue;
        }

        IMediaSample* pSample = NULL;
        if (SUCCEEDED(pin->m_pAllocator->GetBuffer(&pSample, NULL, NULL, 0)) && pSample) {
            BYTE* pData = NULL;
            if (SUCCEEDED(pSample->GetPointer(&pData)) && pData) {
                long sampleSize = pSample->GetSize();
                if (sampleSize > 0) {
                    long copyLen = (long)(std::min)((uint32_t)sampleSize, hdr->payload_size);
                    memcpy(pData, payload, copyLen);
                    pSample->SetActualDataLength(copyLen);
                }
            }

            // Post-copy seqlock validation: verify sequence did not change during copy (anti-tearing guard)
            uint32_t endSeq = hdr->sequence;
            if (startSeq != endSeq) {
                // Buffer was overwritten during copy, discard sample to prevent delivering torn frame
                pSample->Release();
                continue;
            }

            pin->m_lastSequence = endSeq;

            REFERENCE_TIME rtStart = rtTime;
            REFERENCE_TIME rtEnd = rtTime + rtFrameDuration;
            pSample->SetTime(&rtStart, &rtEnd);
            pSample->SetSyncPoint(hdr->is_keyframe ? TRUE : FALSE);

            pin->m_pInputPin->Receive(pSample);
            pSample->Release();
            rtTime += rtFrameDuration;
        }
    }
    return 0;
}

// ----------------------------------------------------------------------------
// Simple Single Pin Enumerator
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

STDMETHODIMP LinkCamFilter::EnumPins(IEnumPins** ppEnum) {
    if (!ppEnum) return E_POINTER;
    *ppEnum = new SinglePinEnum(m_pPin);
    return S_OK;
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
