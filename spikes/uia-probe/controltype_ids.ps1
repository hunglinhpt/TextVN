# SPDX-License-Identifier: GPL-3.0-or-later
# Dump id ControlType static cua UIA2 (.NET) - bang ground truth cho uia-spike.md.
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$ct = [System.Windows.Automation.ControlType]
$flags = [System.Reflection.BindingFlags]::Public -bor [System.Reflection.BindingFlags]::Static -bor [System.Reflection.BindingFlags]::FlattenHierarchy
$names = New-Object System.Collections.Generic.List[string]
foreach ($m in $ct.GetMembers($flags)) {
    if ($m.MemberType -eq [System.Reflection.MemberTypes]::Property) {
        try {
            $pi = $ct.GetProperty($m.Name, $flags)
            if ($pi -and $pi.PropertyType.Name -eq 'ControlType') {
                $v = $pi.GetValue($null, $null)
                if ($v) { [void]$names.Add($m.Name); Write-Output ($m.Name + " = " + $v.Id) }
            }
        } catch { }
    } elseif ($m.MemberType -eq [System.Reflection.MemberTypes]::Field) {
        try {
            $fi = $ct.GetField($m.Name, $flags)
            if ($fi -and $fi.FieldType.Name -eq 'ControlType') {
                $v = $fi.GetValue($null)
                if ($v) { [void]$names.Add($m.Name); Write-Output ($m.Name + " = " + $v.Id) }
            }
        } catch { }
    }
}
Write-Output ("count=" + $names.Count)
