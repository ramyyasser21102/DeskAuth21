import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { browserStorage } from "@/features/theme/theme-preference";
import { parseProfile, serializeProfile, type Profile } from "./profile";

export const PROFILE_STORAGE_KEY = "deskauth21.profile";

interface ProfileContextValue {
  profile: Profile;
  updateProfile: (changes: Partial<Profile>) => void;
}

const ProfileContext = createContext<ProfileContextValue | null>(null);

function loadProfile(): Profile {
  try {
    return parseProfile(browserStorage.getItem(PROFILE_STORAGE_KEY));
  } catch {
    return parseProfile(null);
  }
}

function saveProfile(profile: Profile): void {
  try {
    browserStorage.setItem(PROFILE_STORAGE_KEY, serializeProfile(profile));
  } catch {
    // Saving is a convenience; the change still shows for this session.
  }
}

export function ProfileProvider({ children }: { children: ReactNode }) {
  const [profile, setProfile] = useState(loadProfile);
  const updateProfile = useCallback((changes: Partial<Profile>) => {
    setProfile((current) => {
      const next = { ...current, ...changes };
      saveProfile(next);
      return next;
    });
  }, []);
  const value = useMemo(() => ({ profile, updateProfile }), [profile, updateProfile]);
  return <ProfileContext.Provider value={value}>{children}</ProfileContext.Provider>;
}

export function useProfile(): ProfileContextValue {
  const context = useContext(ProfileContext);
  if (!context) throw new Error("useProfile must be used inside <ProfileProvider>");
  return context;
}
