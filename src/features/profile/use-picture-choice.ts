import { useState } from "react";
import { useProfile } from "./profile-provider";
import { resizeToAvatar } from "./resize-picture";

/** Turns a chosen image file into the saved avatar, or a message saying why it could not. */
export function usePictureChoice() {
  const { updateProfile } = useProfile();
  const [problem, setProblem] = useState<string | null>(null);

  const choosePicture = async (file: File | undefined) => {
    if (!file) return;
    try {
      updateProfile({ pictureDataUrl: await resizeToAvatar(file) });
      setProblem(null);
    } catch (error) {
      setProblem(`Could not use that image (${error instanceof Error ? error.message : String(error)}).`);
    }
  };
  const removePicture = () => updateProfile({ pictureDataUrl: null });

  return { problem, choosePicture, removePicture };
}
