import { Check } from "lucide-react";
import { motion } from "motion/react";
import { REVEAL_GROUP, REVEAL_ITEM } from "@/lib/motion-tokens";
import { AppLogo } from "@/components/app-logo";
import { Button } from "@/components/ui/button";
import { SECURITY_POINTS, WELCOME_STEPS, WELCOME_SUMMARY, type WelcomePoint } from "./welcome-content";

function PointList({ heading, points }: { heading: string; points: readonly WelcomePoint[] }) {
  return (
    <motion.section variants={REVEAL_ITEM} className="flex flex-col gap-3">
      <h3 className="text-sm font-semibold">{heading}</h3>
      <motion.ul variants={REVEAL_GROUP} className="flex flex-col gap-3">
        {points.map(({ title, detail }) => (
          <motion.li key={title} variants={REVEAL_ITEM} className="flex gap-3">
            <Check className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
            <p className="text-sm"><span className="font-medium">{title}.</span> <span className="text-muted-foreground">{detail}</span></p>
          </motion.li>
        ))}
      </motion.ul>
    </motion.section>
  );
}

export function WelcomePage({ onContinue, continueLabel }: { onContinue: () => void; continueLabel: string }) {
  return (
    <motion.div variants={REVEAL_GROUP} initial="hidden" animate="show" className="flex flex-col gap-6">
      <motion.div variants={REVEAL_ITEM} className="flex flex-col items-center gap-3 text-center">
        <AppLogo className="size-16" />
        <h2 className="text-2xl font-semibold tracking-tight">Welcome to DA21</h2>
        <p className="text-sm text-muted-foreground">{WELCOME_SUMMARY}</p>
      </motion.div>
      <PointList heading="How it works" points={WELCOME_STEPS} />
      <PointList heading="How your codes stay safe" points={SECURITY_POINTS} />
      <motion.div variants={REVEAL_ITEM} className="flex flex-col"><Button onClick={onContinue}>{continueLabel}</Button></motion.div>
    </motion.div>
  );
}
