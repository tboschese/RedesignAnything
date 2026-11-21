import { useState } from "react";
import { Button, TextField, color, space } from "@fixture/acme-ds";

type Step = 1 | 2 | 3;
const APPROVAL_LIMIT = 500;
const STEPS = ["Claim details", "Approval", "Review"];

/** A viewer may look at a claim but not submit it: `?role=viewer`. */
const viewer = new URLSearchParams(location.search).get("role") === "viewer";

export function App() {
  const [step, setStep] = useState<Step>(1);
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState("");
  const [approver, setApprover] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [submitted, setSubmitted] = useState(false);
  const value = Number(amount.replace(",", "."));
  const needsApprover = value > APPROVAL_LIMIT;

  const fromDetails = () => {
    const e: Record<string, string> = {};
    if (!description.trim()) e.description = "Enter a description.";
    if (!(value > 0)) e.amount = "Enter an amount greater than 0.";
    setErrors(e);
    if (Object.keys(e).length === 0) setStep(2);
  };
  const fromApproval = () => {
    if (needsApprover && !approver.trim()) {
      setErrors({ approver: `Claims over ${APPROVAL_LIMIT} need an approver code.` });
      return;
    }
    setErrors({});
    setStep(3);
  };

  return (
    <div style={{ minHeight: "100vh", background: color.canvas, color: color.text, fontFamily: "system-ui, sans-serif" }}>
      <header style={{ background: color.surface, borderBottom: `1px solid ${color.border}`, padding: `${space.md} ${space.lg}` }}>
        <p style={{ margin: 0, fontWeight: 700 }}>Acme Expenses</p>
        <ol aria-label="Progress" style={{ display: "flex", gap: space.lg, margin: `${space.sm} 0 0`, padding: 0, listStyle: "none" }}>
          {STEPS.map((name, i) => (
            <li key={name} aria-current={step === i + 1 ? "step" : undefined} style={{ color: step === i + 1 ? color.brandPrimary : color.muted, fontWeight: step === i + 1 ? 700 : 400 }}>
              {i + 1}. {name}
            </li>
          ))}
        </ol>
      </header>
      <main style={{ maxWidth: 560, margin: `${space.lg} auto`, background: color.surface, padding: space.lg, border: `1px solid ${color.border}` }}>
        {submitted ? (
          <section>
            <h1 role="status">Claim submitted.</h1>
            <p style={{ color: color.muted }}>We will email you when it is approved.</p>
          </section>
        ) : step === 1 ? (
          <section>
            <h1>Claim details</h1>
            <TextField label="Description" required value={description} onChange={setDescription} error={errors.description} />
            <TextField label="Amount (EUR)" required value={amount} onChange={setAmount} error={errors.amount} />
            <Button onClick={fromDetails}>Continue</Button>
          </section>
        ) : step === 2 ? (
          <section>
            <h1>Approval</h1>
            <p>Amount: {value.toFixed(2)} EUR</p>
            {needsApprover ? (
              <TextField label="Approver code" required value={approver} onChange={setApprover} error={errors.approver} />
            ) : (
              <p style={{ color: color.muted }}>Claims up to {APPROVAL_LIMIT} EUR need no approver.</p>
            )}
            <div style={{ display: "flex", gap: space.sm }}>
              <Button variant="secondary" onClick={() => { setErrors({}); setStep(1); }}>Back</Button>
              <Button onClick={fromApproval}>Continue</Button>
            </div>
          </section>
        ) : (
          <section>
            <h1>Review</h1>
            <dl>
              <dt>Description</dt><dd>{description}</dd>
              <dt>Amount</dt><dd>{value.toFixed(2)} EUR</dd>
              {needsApprover ? (<><dt>Approver code</dt><dd>{approver}</dd></>) : null}
            </dl>
            {viewer ? <p role="alert" style={{ color: color.danger }}>Viewers cannot submit claims.</p> : null}
            <div style={{ display: "flex", gap: space.sm }}>
              <Button variant="secondary" onClick={() => setStep(2)}>Back</Button>
              <Button disabled={viewer} onClick={() => setSubmitted(true)}>Submit claim</Button>
            </div>
          </section>
        )}
      </main>
    </div>
  );
}
