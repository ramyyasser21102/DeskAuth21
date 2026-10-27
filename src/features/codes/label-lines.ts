export interface LabelLines {
  primary: string;
  /** The account name under the service; `null` when there is no service to put it under. */
  secondary: string | null;
}

/** The service is what people look for, so it leads and the account name sits beneath it. */
export function labelLines(issuer: string, name: string): LabelLines {
  return issuer === "" ? { primary: name, secondary: null } : { primary: issuer, secondary: name };
}
