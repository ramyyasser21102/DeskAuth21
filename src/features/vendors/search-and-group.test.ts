import { describe, expect, it } from "vitest";
import type { CodeView, VendorId, VendorInfo } from "@/lib/api";
import { groupByVendor, matchesSearch } from "./search-and-group";

const VENDORS: VendorInfo[] = [
  { id: "aegis", name: "Aegis" },
  { id: "authy", name: "Authy" },
  { id: "google_authenticator", name: "Google Authenticator" },
  { id: "other", name: "Other" },
];

const account = (issuer: string, name: string, vendor: VendorId): CodeView => ({
  id: `${issuer}-${name}`, label: `${issuer} (${name})`, issuer, name, vendor, code: "123456", seconds_left: 10, period_seconds: 30,
});

describe("matchesSearch", () => {
  const github = account("GitHub", "ramy@example.com", "aegis");

  it("matches the service, the account name or the app, in any case", () => {
    expect(matchesSearch(github, "git", "Aegis")).toBe(true);
    expect(matchesSearch(github, "RAMY@", "Aegis")).toBe(true);
    expect(matchesSearch(github, "aeg", "Aegis")).toBe(true);
  });

  it("needs every word to match, in any order", () => {
    expect(matchesSearch(github, "aegis hub", "Aegis")).toBe(true);
    expect(matchesSearch(github, "hub gitlab", "Aegis")).toBe(false);
  });

  it("matches everything when nothing is typed", () => {
    expect(matchesSearch(github, "", "Aegis")).toBe(true);
    expect(matchesSearch(github, "   ", "Aegis")).toBe(true);
  });
});

describe("groupByVendor", () => {
  const views = [
    account("Zoom", "z@x", "other"),
    account("Slack", "s@x", "authy"),
    account("Amazon", "a@x", "authy"),
    account("Google", "g@x", "google_authenticator"),
    account("GitHub", "h@x", "aegis"),
  ];

  it("orders apps A to Z with Other last", () => {
    expect(groupByVendor(views, VENDORS).map((group) => group.name)).toEqual(["Aegis", "Authy", "Google Authenticator", "Other"]);
  });

  it("orders the accounts inside a group A to Z", () => {
    const authy = groupByVendor(views, VENDORS).find((group) => group.vendor === "authy");
    expect(authy?.views.map((view) => view.issuer)).toEqual(["Amazon", "Slack"]);
  });

  it("leaves out apps that have no accounts and copes with none at all", () => {
    expect(groupByVendor([views[0]], VENDORS)).toHaveLength(1);
    expect(groupByVendor([], VENDORS)).toEqual([]);
  });
});
