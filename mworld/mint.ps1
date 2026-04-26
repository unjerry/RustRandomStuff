@(
    Get-Item Cargo.toml
    Get-ChildItem src -Recurse -Include *.rs,*.wgsl
) | ForEach-Object {
    "===== $($_.FullName) ====="
    Get-Content $_.FullName
    ""
} > rust_project_export.txt

"===== assets files =====" >> rust_project_export.txt

if (Test-Path assets) {
    Get-ChildItem assets -Recurse -File | ForEach-Object {
        $_.FullName
    } >> rust_project_export.txt
}