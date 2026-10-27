import { useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MAX_NAME_LENGTH } from "./profile";
import { ProfileAvatar } from "./profile-avatar";
import { useProfile } from "./profile-provider";
import { usePictureChoice } from "./use-picture-choice";

const NAME_FIELD_ID = "profile-name";
const PRIVACY_NOTE = "Stays on this computer only: it is not stored in your vault and never synced.";

export function ProfileSetting() {
  const { profile, updateProfile } = useProfile();
  const { problem, choosePicture, removePicture } = usePictureChoice();
  const pickerRef = useRef<HTMLInputElement>(null);

  return (
    <div className="flex flex-col gap-3">
      <span className="text-sm font-medium">Profile</span>
      <div className="flex items-center gap-3">
        <ProfileAvatar profile={profile} className="size-14 text-lg" />
        <Button type="button" size="sm" variant="outline" onClick={() => pickerRef.current?.click()}>Choose picture…</Button>
        {profile.pictureDataUrl && <Button type="button" size="sm" variant="ghost" onClick={removePicture}>Remove</Button>}
        <input ref={pickerRef} type="file" accept="image/*" aria-label="Profile picture" className="hidden" onChange={(event) => void choosePicture(event.target.files?.[0])} />
      </div>
      <Label htmlFor={NAME_FIELD_ID}>Your name</Label>
      <Input id={NAME_FIELD_ID} value={profile.name} maxLength={MAX_NAME_LENGTH} autoComplete="off" onChange={(event) => updateProfile({ name: event.target.value })} />
      <p className="text-xs text-muted-foreground">{problem ?? PRIVACY_NOTE}</p>
    </div>
  );
}
