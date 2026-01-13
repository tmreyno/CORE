> Generated test output from `src-tauri/examples/test_report.rs`. Do not edit by hand.\n+\n+# Digital Forensic Examination Report

**Report Number:** FR-2026-0001  
**Version:** 1.0  
**Classification:** LawEnforcementSensitive  
**Generated:** 2026-01-04 04:19:50 UTC

---

## Case Information

| Field | Value |
|-------|-------|
| **Case Number** | 2026-CF-00123 |
| **Case Name** | State v. John Doe |
| **Agency** | Metro Police Department |
| **Requestor** | Det. Jane Smith |
| **Examination Start** | 2026-01-04 |
| **Investigation Type** | Fraud Investigation |


### Examiner

| Field | Value |
|-------|-------|
| **Name** | Alex Johnson |
| **Title** | Senior Digital Forensic Examiner |
| **Organization** | Metro Police Forensic Lab |
| **Email** | ajohnson@metro.gov |
| **Phone** | (555) 123-4567 |
| **Badge Number** | F-1234 |



**Certifications:** EnCE (EnCase Certified Examiner), GCFE (GIAC Certified Forensic Examiner), ACE (AccessData Certified Examiner)


---


## Executive Summary

This report documents the forensic examination of digital evidence seized pursuant to Search Warrant #2026-SW-456. The examination revealed several artifacts of evidentiary value including documents, communications, and internet history relevant to the investigation. Key findings include evidence of document manipulation and communication records between subjects of interest.

---



## Scope of Examination

The scope of this examination was limited to the recovery and analysis of digital artifacts from the submitted evidence items. This examination focused on document recovery, communication analysis, and timeline reconstruction.

---



## Methodology

Standard forensic methodology was employed throughout this examination:
1. Evidence intake and chain of custody documentation
2. Forensic imaging using write-blocking hardware
3. Hash verification of forensic images
4. Systematic artifact extraction and analysis
5. Documentation and report generation

---



## Evidence Examined

| ID | Description | Type | Serial Number | Capacity |
|----|-------------|------|---------------|----------|
| E001 | Dell Latitude laptop computer | Laptop | SN-DELL-2024-001 | 512 GB SSD |
| E002 | Samsung Galaxy S23 smartphone | MobilePhone | IMEI-123456789012345 | 256 GB |




### Evidence E001 - Hash Values

| Algorithm | Value | Verified |
|-----------|-------|----------|
| MD5 | `d41d8cd98f00b204e9800998ecf8427e` | ✓ |
| SHA256 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | ✓ |






---



## Findings


### F001: Deleted Financial Documents Recovered

**Severity:** High  
**Category:** DeletedData

Analysis of unallocated space revealed 47 deleted Microsoft Excel spreadsheets containing financial records. These documents appear to have been deleted on 2025-12-15, approximately one week before the search warrant was executed. Document metadata indicates they were created by user 'JDoe'.


**Related Files:**

- `/Users/JDoe/Documents/Financials/Q4_2025_Report.xlsx`

- `/Users/JDoe/Documents/Financials/Transactions_Dec.xlsx`




**Timestamps:**

- 2026-01-04 04:19:50




**Notes:** Files recovered using signature-based carving from sectors 0x1A2B3C - 0x1A5F00


---


### F002: Encrypted Communication Application

**Severity:** Medium  
**Category:** Communication

Signal messaging application was installed on both the laptop and mobile device. Message database was recovered from the mobile device backup. Analysis revealed communications with 3 contacts during the relevant time period.







---


### F003: Browser History Analysis

**Severity:** Low  
**Category:** InternetHistory

Chrome browser history shows searches related to 'how to permanently delete files' and 'forensic file recovery' on 2025-12-14, the day before the deleted documents were removed.


**Related Files:**

- `/Users/JDoe/AppData/Local/Google/Chrome/User Data/Default/History`




**Timestamps:**

- 2026-01-04 04:19:50





---




## Timeline of Events

| Timestamp | Type | Description | Source | Artifact |
|-----------|------|-------------|--------|----------|
| 2026-01-04 04:19:50 | File Creation | Q4 Financial Report created | NTFS $MFT | Q4_2025_Report.xlsx |
| 2026-01-04 04:19:50 | Web Search | Search: 'how to permanently delete files' | Chrome History | - |
| 2026-01-04 04:19:50 | File Deletion | 47 Excel files deleted from Documents folder | NTFS $MFT / $UsnJrnl | - |


---



## Hash Verification

| Item | Algorithm | Value | Verified |
|------|-----------|-------|----------|
| E001 - Forensic Image | MD5 | `d41d8cd98f00b204e9800998ecf8427e` | ✓ |
| E001 - Forensic Image | SHA256 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | ✓ |


---



## Tools Used

| Tool | Version | Vendor | Purpose |
|------|---------|--------|---------|
| FTK Imager | 4.7.1.2 | Exterro | Forensic imaging and hash verification |
| EnCase Forensic | 23.4 | OpenText | Evidence analysis and artifact extraction |
| FFX Forensic Toolkit | 0.1.0 | CORE Project | Report generation and timeline analysis |


---



## Conclusions

Based on the forensic examination of the submitted evidence, the following conclusions are supported:

1. Financial documents were systematically deleted from the laptop computer approximately one week before the search warrant was executed.

2. Internet search history indicates the user researched file deletion methods the day before the documents were removed.

3. Encrypted communication applications were in use on both devices during the relevant time period.

These findings are presented for investigative purposes and are subject to further analysis as needed.

---




---

*This report was generated by FFX Forensic Toolkit on 2026-01-04 at 04:19:50 UTC.*

**LAWENFORCEMENTSENSITIVE**
