// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock nowISO to return a deterministic timestamp
// Path is relative to THIS test file: __tests__/../../../../types/project = src/types/project
vi.mock("../../../../types/project", () => ({
  nowISO: () => "2025-01-15T10:00:00.000Z",
}));

// Mock invoke (not used by conversion functions, but imported by module)
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock logger
vi.mock("../../../../utils/logger", () => ({
  logger: { scope: () => ({ info: vi.fn(), warn: vi.fn(), error: vi.fn() }) },
}));

import {
  cocItemToDb,
  dbToCocItem,
  cocTransferToDb,
  dbToCocTransfer,
  evidenceCollectionToDb,
  collectedItemToDb,
  dbToEvidenceCollectionData,
  dbToCollectedItem,
} from "../cocDbSync";
import type { COCItem, COCTransfer, EvidenceCollectionData, CollectedItem } from "../../types";
import type {
  DbCocItem,
  DbCocTransfer,
  DbEvidenceCollection,
  DbCollectedItem,
} from "../../../../types/projectDb";

// =============================================================================
// Test fixtures
// =============================================================================

function makeCocItem(overrides?: Partial<COCItem>): COCItem {
  return {
    id: "coc-1",
    coc_number: "2024-001-COC-001",
    evidence_id: "EV-001",
    case_number: "2024-001",
    description: "Dell Latitude laptop",
    item_type: "Laptop",
    make: "Dell",
    model: "Latitude 5540",
    serial_number: "SN123",
    capacity: "512GB SSD",
    condition: "Good",
    acquisition_date: "2024-06-01",
    entered_custody_date: "2024-06-01",
    submitted_by: "Agent Smith",
    received_by: "Lab Tech A",
    received_location: "Digital Forensics Lab",
    storage_location: "Locker 42",
    reason_submitted: "Investigation",
    transfers: [],
    intake_hashes: [{ item: "EV-001", algorithm: "MD5", value: "abc123" }],
    notes: "Some notes",
    ...overrides,
  };
}

function makeDbCocItem(overrides?: Partial<DbCocItem>): DbCocItem {
  return {
    id: "coc-1",
    cocNumber: "2024-001-COC-001",
    evidenceId: "EV-001",
    caseNumber: "2024-001",
    description: "Dell Latitude laptop",
    itemType: "Laptop",
    make: "Dell",
    model: "Latitude 5540",
    serialNumber: "SN123",
    capacity: "512GB SSD",
    condition: "Good",
    acquisitionDate: "2024-06-01",
    enteredCustodyDate: "2024-06-01",
    submittedBy: "Agent Smith",
    receivedBy: "Lab Tech A",
    receivedLocation: "Digital Forensics Lab",
    storageLocation: "Locker 42",
    reasonSubmitted: "Investigation",
    intakeHashesJson: JSON.stringify([{ algorithm: "MD5", value: "abc123" }]),
    notes: "Some notes",
    status: "draft",
    createdAt: "2025-01-15T10:00:00.000Z",
    modifiedAt: "2025-01-15T10:00:00.000Z",
    ...overrides,
  };
}

function makeCocTransfer(overrides?: Partial<COCTransfer>): COCTransfer {
  return {
    id: "xfer-1",
    timestamp: "2024-06-15T09:00:00Z",
    released_by: "Lab Tech A",
    received_by: "Examiner B",
    purpose: "Analysis",
    location: "Lab 1",
    method: "in-person",
    notes: "Transfer notes",
    ...overrides,
  };
}

function makeDbCocTransfer(overrides?: Partial<DbCocTransfer>): DbCocTransfer {
  return {
    id: "xfer-1",
    cocItemId: "coc-1",
    timestamp: "2024-06-15T09:00:00Z",
    releasedBy: "Lab Tech A",
    receivedBy: "Examiner B",
    purpose: "Analysis",
    location: "Lab 1",
    method: "in-person",
    notes: "Transfer notes",
    ...overrides,
  };
}

function makeCollectedItem(overrides?: Partial<CollectedItem>): CollectedItem {
  return {
    id: "item-1",
    item_number: "001",
    description: "Desktop computer",
    device_type: "desktop_computer",
    storage_interface: "sata",
    make: "Dell",
    model: "OptiPlex 7090",
    serial_number: "SN456",
    condition: "good",
    packaging: "Evidence bag",
    building: "HQ",
    room: "202A",
    location_other: "Desk 5",
    notes: "Item notes",
    photo_refs: ["Photo 1", "Photo 2"],
    ...overrides,
  };
}

// =============================================================================
// cocItemToDb
// =============================================================================

describe("cocItemToDb", () => {
  it("converts all required fields", () => {
    const db = cocItemToDb(makeCocItem());
    expect(db.id).toBe("coc-1");
    expect(db.cocNumber).toBe("2024-001-COC-001");
    expect(db.evidenceId).toBe("EV-001");
    expect(db.caseNumber).toBe("2024-001");
    expect(db.description).toBe("Dell Latitude laptop");
    expect(db.itemType).toBe("Laptop");
    expect(db.condition).toBe("Good");
  });

  it("uses nowISO for timestamps", () => {
    const db = cocItemToDb(makeCocItem());
    expect(db.createdAt).toBe("2025-01-15T10:00:00.000Z");
    expect(db.modifiedAt).toBe("2025-01-15T10:00:00.000Z");
  });

  it("defaults status to draft", () => {
    const db = cocItemToDb(makeCocItem());
    expect(db.status).toBe("draft");
  });

  it("preserves explicit status", () => {
    const db = cocItemToDb(makeCocItem({ status: "locked", locked_at: "2025-01-01", locked_by: "JD" }));
    expect(db.status).toBe("locked");
    expect(db.lockedAt).toBe("2025-01-01");
    expect(db.lockedBy).toBe("JD");
  });

  it("serializes intake_hashes to JSON", () => {
    const db = cocItemToDb(makeCocItem({ intake_hashes: [{ item: "EV-001", algorithm: "SHA256", value: "def456" }] }));
    expect(db.intakeHashesJson).toBe(JSON.stringify([{ item: "EV-001", algorithm: "SHA256", value: "def456" }]));
  });

  it("sets intakeHashesJson to undefined when empty", () => {
    const db = cocItemToDb(makeCocItem({ intake_hashes: [] }));
    expect(db.intakeHashesJson).toBeUndefined();
  });

  it("uses empty string fallback for NOT NULL fields", () => {
    const db = cocItemToDb(makeCocItem({
      case_number: "",
      evidence_id: "",
      submitted_by: "",
      received_by: "",
    }));
    expect(db.caseNumber).toBe("");
    expect(db.evidenceId).toBe("");
    expect(db.submittedBy).toBe("");
    expect(db.receivedBy).toBe("");
  });
});

// =============================================================================
// dbToCocItem
// =============================================================================

describe("dbToCocItem", () => {
  it("converts all required fields", () => {
    const item = dbToCocItem(makeDbCocItem());
    expect(item.id).toBe("coc-1");
    expect(item.coc_number).toBe("2024-001-COC-001");
    expect(item.evidence_id).toBe("EV-001");
    expect(item.case_number).toBe("2024-001");
    expect(item.description).toBe("Dell Latitude laptop");
  });

  it("always returns empty transfers array", () => {
    const item = dbToCocItem(makeDbCocItem());
    expect(item.transfers).toEqual([]);
  });

  it("parses intakeHashesJson", () => {
    const item = dbToCocItem(makeDbCocItem({ intakeHashesJson: '[{"algorithm":"MD5","value":"abc"}]' }));
    expect(item.intake_hashes).toEqual([{ algorithm: "MD5", value: "abc" }]);
  });

  it("returns empty array when intakeHashesJson is undefined", () => {
    const item = dbToCocItem(makeDbCocItem({ intakeHashesJson: undefined }));
    expect(item.intake_hashes).toEqual([]);
  });

  it("defaults item_type to HardDrive", () => {
    const item = dbToCocItem(makeDbCocItem({ itemType: "" }));
    expect(item.item_type).toBe("HardDrive");
  });
});

// =============================================================================
// cocTransferToDb
// =============================================================================

describe("cocTransferToDb", () => {
  it("converts all fields and adds cocItemId", () => {
    const db = cocTransferToDb(makeCocTransfer(), "parent-coc-1");
    expect(db.id).toBe("xfer-1");
    expect(db.cocItemId).toBe("parent-coc-1");
    expect(db.timestamp).toBe("2024-06-15T09:00:00Z");
    expect(db.releasedBy).toBe("Lab Tech A");
    expect(db.receivedBy).toBe("Examiner B");
    expect(db.purpose).toBe("Analysis");
    expect(db.location).toBe("Lab 1");
    expect(db.method).toBe("in-person");
    expect(db.notes).toBe("Transfer notes");
  });
});

// =============================================================================
// dbToCocTransfer
// =============================================================================

describe("dbToCocTransfer", () => {
  it("converts all fields back to wizard type", () => {
    const transfer = dbToCocTransfer(makeDbCocTransfer());
    expect(transfer.id).toBe("xfer-1");
    expect(transfer.timestamp).toBe("2024-06-15T09:00:00Z");
    expect(transfer.released_by).toBe("Lab Tech A");
    expect(transfer.received_by).toBe("Examiner B");
    expect(transfer.purpose).toBe("Analysis");
    expect(transfer.location).toBe("Lab 1");
    expect(transfer.method).toBe("in-person");
    expect(transfer.notes).toBe("Transfer notes");
  });
});

// =============================================================================
// Round-trip: COCItem → DbCocItem → COCItem
// =============================================================================

describe("COCItem round-trip", () => {
  it("preserves data through cocItemToDb → dbToCocItem", () => {
    const original = makeCocItem();
    const db = cocItemToDb(original);
    const restored = dbToCocItem(db);

    expect(restored.id).toBe(original.id);
    expect(restored.coc_number).toBe(original.coc_number);
    expect(restored.evidence_id).toBe(original.evidence_id);
    expect(restored.description).toBe(original.description);
    expect(restored.intake_hashes).toEqual(original.intake_hashes);
    expect(restored.transfers).toEqual([]); // transfers loaded separately
  });
});

// =============================================================================
// Round-trip: COCTransfer → DbCocTransfer → COCTransfer
// =============================================================================

describe("COCTransfer round-trip", () => {
  it("preserves data through cocTransferToDb → dbToCocTransfer", () => {
    const original = makeCocTransfer();
    const db = cocTransferToDb(original, "coc-99");
    const restored = dbToCocTransfer(db);

    expect(restored.id).toBe(original.id);
    expect(restored.timestamp).toBe(original.timestamp);
    expect(restored.released_by).toBe(original.released_by);
    expect(restored.received_by).toBe(original.received_by);
    expect(restored.purpose).toBe(original.purpose);
    expect(restored.location).toBe(original.location);
    expect(restored.method).toBe(original.method);
    expect(restored.notes).toBe(original.notes);
  });
});

// =============================================================================
// collectedItemToDb
// =============================================================================

describe("collectedItemToDb", () => {
  it("converts required fields", () => {
    const db = collectedItemToDb(makeCollectedItem(), "col-1");
    expect(db.id).toBe("item-1");
    expect(db.collectionId).toBe("col-1");
    expect(db.itemNumber).toBe("001");
    expect(db.description).toBe("Desktop computer");
    expect(db.condition).toBe("good");
    expect(db.packaging).toBe("Evidence bag");
  });

  it("composes foundLocation from building/room/location_other", () => {
    const db = collectedItemToDb(makeCollectedItem(), "col-1");
    expect(db.foundLocation).toBe("HQ, 202A, Desk 5");
  });

  it("handles partial location fields", () => {
    const db = collectedItemToDb(makeCollectedItem({ building: "HQ", room: undefined, location_other: undefined }), "col-1");
    expect(db.foundLocation).toBe("HQ");
  });

  it("handles empty location fields", () => {
    const db = collectedItemToDb(makeCollectedItem({ building: undefined, room: undefined, location_other: undefined }), "col-1");
    expect(db.foundLocation).toBe("");
  });

  it("serializes photo_refs to JSON", () => {
    const db = collectedItemToDb(makeCollectedItem({ photo_refs: ["A", "B"] }), "col-1");
    expect(db.photoRefsJson).toBe(JSON.stringify(["A", "B"]));
  });

  it("sets photoRefsJson to undefined when empty", () => {
    const db = collectedItemToDb(makeCollectedItem({ photo_refs: [] }), "col-1");
    expect(db.photoRefsJson).toBeUndefined();
  });

  it("passes through optional FKs", () => {
    const db = collectedItemToDb(makeCollectedItem(), "col-1", "coc-99", "ev-file-1");
    expect(db.cocItemId).toBe("coc-99");
    expect(db.evidenceFileId).toBe("ev-file-1");
  });

  it("maps device_type 'other' to device_type_other value", () => {
    const db = collectedItemToDb(makeCollectedItem({ device_type: "other", device_type_other: "Custom Device" }), "col-1");
    expect(db.itemType).toBe("Custom Device");
  });
});

// =============================================================================
// evidenceCollectionToDb
// =============================================================================

describe("evidenceCollectionToDb", () => {
  function makeEvidenceCollection(): EvidenceCollectionData {
    return {
      collection_date: "2024-06-01",
      collecting_officer: "Agent Smith",
      authorization: "Warrant #1234",
      authorization_date: "2024-05-31",
      authorizing_authority: "Judge Jones",
      witnesses: ["Witness A", "Witness B"],
      collected_items: [makeCollectedItem()],
      documentation_notes: "Photo log attached",
      conditions: "Office environment",
    };
  }

  it("creates collection and items", () => {
    const { collection, items } = evidenceCollectionToDb(makeEvidenceCollection(), "col-1", "C-001", "draft");
    expect(collection.id).toBe("col-1");
    expect(collection.caseNumber).toBe("C-001");
    expect(collection.collectingOfficer).toBe("Agent Smith");
    expect(collection.status).toBe("draft");
    expect(items).toHaveLength(1);
    expect(items[0].collectionId).toBe("col-1");
  });

  it("uses nowISO for timestamps", () => {
    const { collection } = evidenceCollectionToDb(makeEvidenceCollection(), "col-1");
    expect(collection.createdAt).toBe("2025-01-15T10:00:00.000Z");
    expect(collection.modifiedAt).toBe("2025-01-15T10:00:00.000Z");
  });

  it("serializes witnesses to JSON", () => {
    const { collection } = evidenceCollectionToDb(makeEvidenceCollection(), "col-1");
    expect(collection.witnessesJson).toBe(JSON.stringify(["Witness A", "Witness B"]));
  });

  it("sets witnessesJson to undefined when empty", () => {
    const data = makeEvidenceCollection();
    data.witnesses = [];
    const { collection } = evidenceCollectionToDb(data, "col-1");
    expect(collection.witnessesJson).toBeUndefined();
  });

  it("defaults status to draft and caseNumber to empty string", () => {
    const { collection } = evidenceCollectionToDb(makeEvidenceCollection(), "col-1");
    expect(collection.status).toBe("draft");
    expect(collection.caseNumber).toBe("");
  });
});

// =============================================================================
// dbToEvidenceCollectionData
// =============================================================================

describe("dbToEvidenceCollectionData", () => {
  function makeDbCollection(): DbEvidenceCollection {
    return {
      id: "col-1",
      caseNumber: "C-001",
      collectionDate: "2024-06-01",
      collectionLocation: "",
      collectingOfficer: "Agent Smith",
      authorization: "Warrant #1234",
      authorizationDate: "2024-05-31",
      authorizingAuthority: "Judge Jones",
      witnessesJson: JSON.stringify(["Witness A"]),
      documentationNotes: "Notes",
      conditions: "Office",
      status: "draft",
      createdAt: "2025-01-15T10:00:00.000Z",
      modifiedAt: "2025-01-15T10:00:00.000Z",
    };
  }

  it("converts collection fields", () => {
    const data = dbToEvidenceCollectionData(makeDbCollection(), []);
    expect(data.collection_date).toBe("2024-06-01");
    expect(data.collecting_officer).toBe("Agent Smith");
    expect(data.authorization).toBe("Warrant #1234");
    expect(data.documentation_notes).toBe("Notes");
    expect(data.conditions).toBe("Office");
  });

  it("parses witnesses from JSON", () => {
    const data = dbToEvidenceCollectionData(makeDbCollection(), []);
    expect(data.witnesses).toEqual(["Witness A"]);
  });

  it("returns empty witnesses when witnessesJson is undefined", () => {
    const col = makeDbCollection();
    col.witnessesJson = undefined;
    const data = dbToEvidenceCollectionData(col, []);
    expect(data.witnesses).toEqual([]);
  });

  it("converts items via dbToCollectedItem", () => {
    const dbItem: DbCollectedItem = {
      id: "item-1",
      collectionId: "col-1",
      itemNumber: "001",
      description: "Test",
      foundLocation: "Room 1",
      itemType: "desktop_computer",
      condition: "good",
      packaging: "Bag",
    };
    const data = dbToEvidenceCollectionData(makeDbCollection(), [dbItem]);
    expect(data.collected_items).toHaveLength(1);
    expect(data.collected_items[0].id).toBe("item-1");
  });
});

// =============================================================================
// dbToCollectedItem
// =============================================================================

describe("dbToCollectedItem", () => {
  it("uses structured location fields when available", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "Legacy Location",
      itemType: "desktop_computer",
      condition: "good",
      packaging: "",
      building: "HQ",
      room: "101",
      locationOther: "Desk 3",
    });
    expect(item.building).toBe("HQ");
    expect(item.room).toBe("101");
    expect(item.location_other).toBe("Desk 3");
  });

  it("parses legacy comma-separated foundLocation when no structured fields", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "Building A, Room 5, Corner desk",
      itemType: "desktop_computer",
      condition: "good",
      packaging: "",
    });
    expect(item.building).toBe("Building A");
    expect(item.room).toBe("Room 5");
    expect(item.location_other).toBe("Corner desk");
  });

  it("handles empty foundLocation in legacy mode", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "",
      itemType: "desktop_computer",
      condition: "good",
      packaging: "",
    });
    expect(item.building).toBeUndefined();
    expect(item.room).toBeUndefined();
    expect(item.location_other).toBeUndefined();
  });

  it("parses photoRefsJson", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "",
      itemType: "",
      condition: "good",
      packaging: "",
      photoRefsJson: '["Photo1","Photo2"]',
    });
    expect(item.photo_refs).toEqual(["Photo1", "Photo2"]);
  });

  it("returns empty photo_refs when photoRefsJson is undefined", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "",
      itemType: "",
      condition: "good",
      packaging: "",
    });
    expect(item.photo_refs).toEqual([]);
  });

  it("defaults device_type to desktop_computer when empty", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "",
      itemType: "",
      condition: "good",
      packaging: "",
    });
    expect(item.device_type).toBe("desktop_computer");
  });

  it("prefers deviceType over itemType when both present", () => {
    const item = dbToCollectedItem({
      id: "i1",
      collectionId: "c1",
      itemNumber: "001",
      description: "PC",
      foundLocation: "",
      itemType: "legacy_type",
      condition: "good",
      packaging: "",
      deviceType: "laptop",
    });
    expect(item.device_type).toBe("laptop");
  });
});
