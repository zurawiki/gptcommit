
Param
(
    [parameter(Mandatory = $true)]
    [string]
    $Version,
    [parameter(Mandatory = $false)]
    [string]
    $Token
)

function Get-HashForWindowsZip {
    param (
        # Parameter help description
        [Parameter(Mandatory = $true)]
        [string]
        $Version
    )
    $Url = "https://github.com/zurawiki/gptcommit/releases/download/v$Version/gptcommit-x86_64-pc-windows-msvc.zip"
    $FilePath = "$($env:TEMP)\$(Split-Path -Leaf $Url)"
    Invoke-WebRequest -Uri $Url -OutFile $FilePath
    $Hash = Get-FileHash $FilePath -Algorithm SHA256 | Select-Object -ExpandProperty Hash
    Remove-Item $FilePath
    return $Hash
}

function Write-MetaData {
    param(
        [Parameter(Mandatory = $true)]
        [string]
        $FileName,
        [Parameter(Mandatory = $true)]
        [string]
        $Version,
        [Parameter(Mandatory = $true)]
        [string]
        $Hash
    )
    $content = Get-Content $FileName -Raw
    $content = $content.Replace('<VERSION>', $Version)
    $content = $content.Replace('<HASH>', $Hash)
    $date = Get-Date -Format "yyyy-MM-dd"
    $content = $content.Replace('<DATE>', $date)
    $content | Out-File -Encoding 'UTF8' "./$Version/$FileName"
}

New-Item -Path $PWD -Name $Version -ItemType "directory"
$Hash = Get-HashForWindowsZip -Version $Version
Get-ChildItem '*.yaml' | ForEach-Object -Process {
    Write-MetaData -FileName $_.Name -Version $Version -Hash $Hash
}
if (-not $Token) {
    return
}

# Download Winget-Create msixbundle, install, and execute update.
$appxBundleFile = "$env:TEMP\wingetcreate.msixbundle"
Invoke-WebRequest https://aka.ms/wingetcreate/latest/msixbundle -OutFile $appxBundleFile
Add-AppxPackage $appxBundleFile

# Create PR
# wingetcreate submit --token <GitHubPersonalAccessToken> <PathToManifest>
wingetcreate submit --token $Token $Version