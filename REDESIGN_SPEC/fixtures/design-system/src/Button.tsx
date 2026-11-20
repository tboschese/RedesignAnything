import type { ReactNode } from "react";

export type ButtonProps = {
  variant?: "primary" | "secondary" | "danger";
  children?: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
};

export function Button(props: ButtonProps) {
  return null;
}
