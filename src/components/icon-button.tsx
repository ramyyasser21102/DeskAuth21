import type { ComponentProps } from "react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface IconButtonProps extends Omit<ComponentProps<typeof Button>, "size" | "aria-label"> {
  /** Verb-first name, used for the tooltip and for screen readers. */
  label: string;
}

/** A square icon-only button whose tooltip shows on hover and keyboard focus. */
export function IconButton({ label, className, children, ...buttonProps }: IconButtonProps) {
  return (
    <TooltipProvider>
    <Tooltip>
      <TooltipTrigger asChild>
        <Button type="button" size="icon" aria-label={label} className={cn("size-11", className)} {...buttonProps}>
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
    </TooltipProvider>
  );
}
