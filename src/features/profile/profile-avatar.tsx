import { User } from "lucide-react";
import { cn } from "@/lib/utils";
import { initialsOf, type Profile } from "./profile";

interface ProfileAvatarProps {
  profile: Profile;
  className?: string;
}

/** The picture if there is one, else initials, else a plain person icon. */
export function ProfileAvatar({ profile, className }: ProfileAvatarProps) {
  const initials = initialsOf(profile.name);
  return (
    <span
      className={cn("inline-flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-full border bg-secondary text-sm font-semibold", className)}
    >
      {profile.pictureDataUrl ? (
        <img src={profile.pictureDataUrl} alt="" className="size-full object-cover" />
      ) : initials ? (
        <span aria-hidden="true">{initials}</span>
      ) : (
        <User className="size-4 text-muted-foreground" aria-hidden="true" />
      )}
    </span>
  );
}
