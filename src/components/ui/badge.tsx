import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-sm border px-1.5 py-0.5 font-mono text-[10px] font-medium uppercase tracking-[0.12em]",
  {
    variants: {
      variant: {
        default: "border-primary/50 bg-primary/10 text-primary",
        secondary: "border-border bg-secondary text-secondary-foreground",
        outline: "border-border text-muted-foreground",
        ready: "border-pass/40 bg-transparent text-pass",
        pass: "border-pass/40 bg-transparent text-pass",
        warn: "border-grade-warn/40 bg-transparent text-grade-warn",
        hold: "border-hold/50 bg-hold/10 text-hold",
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
  const variant = g === "PASS" || g === "READY" ? "pass" : g === "WARN" ? "warn" : g ? "hold" : "outline";
  return <Badge variant={variant}>{g || "IDLE"}</Badge>;
}
