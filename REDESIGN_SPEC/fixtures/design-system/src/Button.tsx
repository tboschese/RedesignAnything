import type { ReactNode } from "react";
import { color, space } from "./tokens";

export type ButtonProps = {
  variant?: "primary" | "secondary" | "danger";
  children?: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
};

const looks = {
  primary: { background: color.brandPrimary, color: color.onBrand, border: color.brandPrimary },
  secondary: { background: color.surface, color: color.brandPrimary, border: color.brandPrimary },
  danger: { background: color.danger, color: color.onBrand, border: color.danger },
};

export function Button(props: ButtonProps) {
  const look = looks[props.variant ?? "primary"];
  return (
    <button
      type="button"
      onClick={props.onClick}
      disabled={props.disabled}
      style={{
        background: look.background,
        color: look.color,
        border: `1px solid ${look.border}`,
        borderRadius: space.xs,
        padding: `${space.sm} ${space.md}`,
        font: "inherit",
        fontWeight: 600,
        cursor: props.disabled ? "not-allowed" : "pointer",
        opacity: props.disabled ? 0.55 : 1,
      }}
    >
      {props.children}
    </button>
  );
}
