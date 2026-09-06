import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex shrink-0 items-center rounded-sm border px-2 py-0.5 font-mono text-[10px] font-semibold uppercase tracking-[0.14em]",
  {
    variants: {
      variant: {
        default: "border-primary bg-primary text-primary-foreground",
        secondary: "border-border bg-secondary text-secondary-foreground",
        outline: "border-foreground/45 bg-secondary text-foreground",
        ready: "border-pass bg-pass text-background",
        pass: "border-pass bg-pass text-background",
        warn: "border-grade-warn bg-grade-warn text-background",
        hold: "border-hold bg-hold text-primary-foreground",
        idle: "border-foreground/50 bg-secondary text-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export function Badge({
  className,
  variant,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & VariantProps<typeof badgeVariants>) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export function GradeBadge({ grade }: { grade?: string | null }) {
  const g = (grade ?? "").toUpperCase();
  const variant =
    g === "PASS" || g === "READY" || g === "FOUND"
      ? "pass"
      : g === "WARN"
        ? "warn"
        : g
          ? "hold"
          : "idle";
  return <Badge variant={variant}>{g || "IDLE"}</Badge>;
}
