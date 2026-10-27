import { Check } from "lucide-react";
import { AppLogo } from "@/components/app-logo";
import { Button } from "@/components/ui/button";
import { SECURITY_POINTS, WELCOME_STEPS, WELCOME_SUMMARY, type WelcomePoint } from "./welcome-content";

function PointList({ heading, points }: { heading: string; points: readonly WelcomePoint[] }) {
  return (
    <section className="flex flex-col gap-3">
      <h3 className="text-sm font-semibold">{heading}</h3>
      <ul className="flex flex-col gap-3">
        {points.map(({ title, detail }) => (
          <li key={title} className="flex gap-3">
            <Check className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
            <p className="text-sm"><span className="font-medium">{title}.</span> <span className="text-muted-foreground">{detail}</span></p>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function WelcomePage({ onContinue, continueLabel }: { onContinue: () => void; continueLabel: string }) {
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col items-center gap-3 text-center">
        <AppLogo className="size-16" />
        <h2 className="text-2xl font-semibold tracking-tight">Welcome to DA21</h2>
        <p className="text-sm text-muted-foreground">{WELCOME_SUMMARY}</p>
      </div>
      <PointList heading="How it works" points={WELCOME_STEPS} />
      <PointList heading="How your codes stay safe" points={SECURITY_POINTS} />
      <Button onClick={onContinue}>{continueLabel}</Button>
    </div>
  );
}
