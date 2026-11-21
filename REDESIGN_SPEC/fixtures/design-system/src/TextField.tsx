import { useId } from "react";
import { color, space } from "./tokens";

export type TextFieldProps = {
  label: string;
  value?: string;
  onChange?: (value: string) => void;
  error?: string;
  required?: boolean;
};

export function TextField(props: TextFieldProps) {
  const id = useId();
  const errorId = `${id}-error`;
  return (
    <div style={{ display: "grid", gap: space.xs, marginBottom: space.md }}>
      <label htmlFor={id} style={{ color: color.text, fontWeight: 600 }}>
        {props.label}
        {props.required ? " *" : ""}
      </label>
      <input
        id={id}
        value={props.value ?? ""}
        required={props.required}
        aria-invalid={props.error ? true : undefined}
        aria-describedby={props.error ? errorId : undefined}
        onChange={(e) => props.onChange?.(e.target.value)}
        style={{
          font: "inherit",
          padding: space.sm,
          border: `1px solid ${props.error ? color.danger : color.border}`,
          borderRadius: space.xs,
          color: color.text,
          background: color.surface,
        }}
      />
      {props.error ? (
        <span id={errorId} role="alert" style={{ color: color.danger }}>
          {props.error}
        </span>
      ) : null}
    </div>
  );
}
