Add-Type -MemberDefinition '[DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr LoadLibraryExW(string p, IntPtr h, uint f);' -Name L -Namespace W
$h = [W.L]::LoadLibraryExW('D:\TextVN-verify\textvn-tsf.dll', [IntPtr]::Zero, 0x00000008)
Write-Host ('held handle=' + $h)
Start-Sleep -Seconds 900
