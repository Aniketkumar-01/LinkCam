# LinkCam microphone setup: installs the virtual audio cable driver if missing,
# names it "LinkCam Mic" (Capture) and "LinkCam Bridge" (Render),
# and leaves the user's default speakers and microphone untouched.
#
# Run with administrator rights.

param (
    [switch]$Silent = $false,
    [string]$DriverDir = "",
    [switch]$Clean = $false,
    [switch]$Uninstall = $false
)

$ErrorActionPreference = "Stop"
if (-not $DriverDir) { $DriverDir = Join-Path $PSScriptRoot "driver" }

function Say([string]$text, [string]$color = "Cyan") {
    if (-not $Silent) { Write-Host "[LinkCam] $text" -ForegroundColor $color }
}

function Finish([int]$code) {
    if (-not $Silent) { Read-Host "Press Enter to close" | Out-Null }
    exit $code
}

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    if ($Silent) { exit 1 }
    Say "Setting up LinkCam audio device needs administrator privileges. Requesting elevation..." "Yellow"
    $cleanArg = if ($Clean) { " -Clean" } else { "" }
    $uninstArg = if ($Uninstall) { " -Uninstall" } else { "" }
    $elevated = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -DriverDir `"$DriverDir`"$cleanArg$uninstArg"
    try {
        $proc = Start-Process powershell.exe -ArgumentList $elevated -Verb RunAs -PassThru -Wait
        exit $proc.ExitCode
    } catch {
        Say "Administrator rights were declined. LinkCam microphone was not installed." "Yellow"
        Finish 1
    }
}

$DriverDevice = "VB-Audio Virtual Cable"
$LinkCamAdapter = "LinkCam"
$EarlierAdapter = "LinkCam Audio"
$EndpointName = "{a45c254e-df1c-4efd-8020-67d146a850e0},2"   # PKEY_Device_DeviceDesc
$AdapterName  = "{b3f8fa53-0004-438e-9003-51a46e139bfc},6"   # PKEY_DeviceInterface_FriendlyName

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

[ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
class MMDeviceEnumerator {}

[ComImport, Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceEnumerator {
    int EnumAudioEndpoints();
    [PreserveSig] int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice device);
}

[ComImport, Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevice {
    int Activate();
    int OpenPropertyStore();
    [PreserveSig] int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
}

[ComImport, Guid("F8679F50-850A-41CF-9C72-430F290290C8"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IPolicyConfig {
    int GetMixFormat(); int GetDeviceFormat(); int ResetDeviceFormat(); int SetDeviceFormat();
    int GetProcessingPeriod(); int SetProcessingPeriod(); int GetShareMode(); int SetShareMode();
    int GetPropertyValue(); int SetPropertyValue();
    [PreserveSig] int SetDefaultEndpoint([MarshalAs(UnmanagedType.LPWStr)] string id, int role);
}

[ComImport, Guid("870AF99C-171D-4F9E-AF0D-E63DF40C2BC9")]
class PolicyConfigClient {}

public static class LinkCamDefaults {
    public static string Get(int flow, int role) {
        IMMDevice device;
        var enumerator = (IMMDeviceEnumerator)new MMDeviceEnumerator();
        if (enumerator.GetDefaultAudioEndpoint(flow, role, out device) != 0 || device == null) return null;
        string id;
        return device.GetId(out id) == 0 ? id : null;
    }
    public static void Set(string id, int role) {
        ((IPolicyConfig)new PolicyConfigClient()).SetDefaultEndpoint(id, role);
    }
}
'@

# 1. Save user's current defaults so they are never disrupted
$saved = @()
foreach ($flow in 0, 1) {
    foreach ($role in 0, 1, 2) {
        $id = [LinkCamDefaults]::Get($flow, $role)
        if ($id) { $saved += [pscustomobject]@{ Id = $id; Role = $role } }
    }
}

function Restore-Defaults {
    foreach ($d in $saved) {
        try { [void][LinkCamDefaults]::Set($d.Id, $d.Role) } catch {}
    }
}

function Rename-Endpoints([string]$flow, [string]$name) {
    $root = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\$flow"
    $found = 0
    Get-ChildItem -Path $root -ErrorAction SilentlyContinue | ForEach-Object {
        $props = Join-Path $_.PSPath "Properties"
        $values = Get-ItemProperty -Path $props -ErrorAction SilentlyContinue
        if ($values -and ($values.$AdapterName -in @($DriverDevice, $LinkCamAdapter, $EarlierAdapter))) {
            Set-ItemProperty -Path $props -Name $EndpointName -Value $name -ErrorAction SilentlyContinue
            Set-ItemProperty -Path $props -Name $AdapterName -Value $LinkCamAdapter -ErrorAction SilentlyContinue
            $check = Get-ItemProperty -Path $props -ErrorAction SilentlyContinue
            if ($check -and $check.$EndpointName -eq $name) {
                $found++
            }
        }
    }
    return $found
}

# 2. Check if virtual audio device is installed
$dev = Get-PnpDevice -Class Media -ErrorAction SilentlyContinue | Where-Object { $_.FriendlyName -eq $DriverDevice }

if ($dev) {
    Say "Virtual Audio Cable device detected. Applying LinkCam endpoint branding..." "Green"
    $c = Rename-Endpoints "Capture" "LinkCam Mic"
    $r = Rename-Endpoints "Render"  "LinkCam Bridge"
    Say "Configured $c capture endpoints ('LinkCam Mic') and $r render endpoints ('LinkCam Bridge')." "Green"

    # Refresh AudioEndpointBuilder
    try {
        Restart-Service -Name "AudioEndpointBuilder" -Force -ErrorAction SilentlyContinue
        Start-Service -Name "Audiosrv" -ErrorAction SilentlyContinue
    } catch {}

    Restore-Defaults
    Say "LinkCam Virtual Audio device successfully configured!" "Green"
    Finish 0
} else {
    Say "Virtual Audio device not yet installed. Looking for driver installer..." "Yellow"
    $setup = Join-Path $DriverDir "VBCABLE_Setup_x64.exe"
    if (Test-Path $setup) {
        Say "Installing driver pack..."
        $proc = Start-Process -FilePath $setup -ArgumentList "-i", "-h" -WorkingDirectory $DriverDir -PassThru
        [void]$proc.WaitForExit(180000)
        Start-Sleep -Seconds 2
        Rename-Endpoints "Capture" "LinkCam Mic"
        Rename-Endpoints "Render"  "LinkCam Bridge"
        Restore-Defaults
        Say "Installed and configured LinkCam Mic." "Green"
        Finish 0
    } else {
        Say "Notice: VBCABLE_Setup_x64.exe not found in $DriverDir. Download VB-Audio Cable or run installer with driver pack." "Yellow"
        Finish 1
    }
}
