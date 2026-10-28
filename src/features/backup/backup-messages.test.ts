import { describe, expect, it } from "vitest";
import { describeBackupSaved, describeExportResult } from "./backup-messages";

describe("backup messages", () => {
  it("says where the backup went and that it stays encrypted", () => {
    expect(describeBackupSaved("/Users/me/b.backup")).toContain("/Users/me/b.backup");
    expect(describeBackupSaved("/x")).toContain("encrypted");
  });

  it("counts exported accounts in singular and plural", () => {
    expect(describeExportResult(1)).toBe("Exported 1 account. The file is not encrypted; keep it private.");
    expect(describeExportResult(3)).toContain("3 accounts");
  });
});
