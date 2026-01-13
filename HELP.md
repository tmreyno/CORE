# CORE-FFX Quick Help

A short, practical guide to the current UI and workflows.

## Getting Started

1) Open a directory (toolbar)
2) Scan for evidence
3) Load metadata ("Load All")
4) Select files to view metadata or hash results

## Toolbar Actions

- Open Directory: pick an evidence root
- Scan: scan the current path
- Recursive: include subfolders
- Hash: compute hashes for selected files
- Load All: fetch metadata for all discovered containers
- Save/Load: project file management
- Report: open the report wizard

## Evidence Formats (Primary)

- AD1
- E01/Ex01
- L01/Lx01
- Raw images (.dd, .raw, .img, .001)
- UFED (.ufd, .ufdr)
- Archives (ZIP/7z/RAR metadata; ZIP extraction)

## File List Navigation

- Arrow keys: move selection
- Enter: open active file/tab
- Space: toggle selection
- Home/End: jump to first/last
- Escape: clear filter

## Hashing Tips

- Choose algorithm from the dropdown (forensic vs fast)
- Hash matches are compared against stored hashes when available
- Incomplete containers will show a warning state

## Reports

Report Wizard steps:

1) Case Information
2) Evidence Selection
3) Findings
4) Preview
5) Export (PDF/DOCX/HTML/Markdown)

## Need More Detail?

- Application Guide: `APP_README.md`
- Code Directory: `CODE_BIBLE.md`
- Backend Docs: `src-tauri/src/README.md`
