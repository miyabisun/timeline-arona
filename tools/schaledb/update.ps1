# Refreshes the SchaleDB snapshot of the students: every student of the JP data
# (released or not, outfits and forms included) with the face icon, for the
# editor's names and faces and the MCP server's `students`. The app reads only
# this snapshot, never the network.
#   powershell -File tools/schaledb/update.ps1
$ErrorActionPreference = 'Stop'
$root = Join-Path $PSScriptRoot '..\..\public\schaledb'
Add-Type -AssemblyName System.Web.Extensions

# The response has no charset, which Windows PowerShell would read as Latin-1;
# ConvertFrom-Json refuses keys that differ only in case, which students.json has.
$bytes = (Invoke-WebRequest -UseBasicParsing 'https://schaledb.com/data/jp/students.json').RawContentStream.ToArray()
$parser = New-Object System.Web.Script.Serialization.JavaScriptSerializer
$parser.MaxJsonLength = [int]::MaxValue
$students = $parser.DeserializeObject([Text.Encoding]::UTF8.GetString($bytes)).Values |
  Sort-Object { [int64]$_.Id }

# One student a line keeps the snapshot diffable when SchaleDB adds students.
$lines = foreach ($s in $students) {
  [ordered]@{
    id = [int64]$s.Id
    name = ($s.Name -replace '[\r\n]+', '')
    # STRIKER (Main) or SPECIAL (Support), the party's two groups.
    squad = if ($s.SquadType -eq 'Support') { 'special' } else { 'striker' }
    released = [bool]$s.IsReleased[0]
  } | ConvertTo-Json -Compress
}
New-Item -ItemType Directory -Force $root | Out-Null
[IO.File]::WriteAllText((Join-Path $root 'students.json'), "[`n" + ($lines -join ",`n") + "`n]`n", (New-Object Text.UTF8Encoding($false)))

# SchaleDB answers a missing picture with an HTML page and status 200, so each
# icon is checked to be a WebP before it is kept.
$dir = Join-Path $root 'students'
New-Item -ItemType Directory -Force $dir | Out-Null
$fetched = 0
foreach ($s in $students) {
  $path = Join-Path $dir "$($s.Id).webp"
  if (Test-Path $path) { continue }
  curl.exe -sSf -o $path "https://schaledb.com/images/student/icon/$($s.Id).webp"
  if ($LASTEXITCODE -ne 0) { throw "failed to fetch the icon of $($s.Id)" }
  $head = [IO.File]::ReadAllBytes($path)[8..11]
  if ([Text.Encoding]::ASCII.GetString($head) -ne 'WEBP') {
    Remove-Item $path
    throw "the icon of $($s.Id) is no WebP"
  }
  $fetched++
}
Write-Host "students: $($students.Count) entries, $fetched new icons"
