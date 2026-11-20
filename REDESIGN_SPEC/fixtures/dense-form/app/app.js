(function () {
  const params = new URLSearchParams(location.search);
  const role = params.get("role") === "approver" ? "approver" : "employee";
  const roleLabel = document.getElementById("role-label");
  const approve = document.getElementById("approve");
  const form = document.getElementById("claim-form");
  const amount = document.getElementById("amount");
  const category = document.getElementById("category");
  const date = document.getElementById("date");
  const submit = document.getElementById("submit");
  const errorEl = document.getElementById("form-error");
  const successEl = document.getElementById("form-success");
  const empty = document.getElementById("attachments-empty");
  const receipt = document.getElementById("receipt");

  roleLabel.textContent = "Role: " + (role === "approver" ? "Approver" : "Employee");
  approve.hidden = role !== "approver";

  function amountValid() {
    const n = Number(amount.value);
    return Number.isFinite(n) && n > 0 && n <= 10000;
  }

  function refresh() {
    const ok = amountValid() && category.value !== "" && date.value !== "";
    submit.disabled = !ok;
  }

  ["input", "change"].forEach((evt) => {
    amount.addEventListener(evt, refresh);
    category.addEventListener(evt, refresh);
    date.addEventListener(evt, refresh);
  });

  receipt.addEventListener("change", () => {
    if (receipt.files && receipt.files.length) {
      empty.textContent = receipt.files[0].name;
    } else {
      empty.textContent = "No attachments yet.";
    }
  });

  form.addEventListener("submit", (e) => {
    e.preventDefault();
    errorEl.hidden = true;
    successEl.hidden = true;
    if (!amountValid()) {
      errorEl.textContent = "Amount must be greater than 0 and at most 10000.";
      errorEl.hidden = false;
      return;
    }
    if (!category.value) {
      errorEl.textContent = "Category is required.";
      errorEl.hidden = false;
      return;
    }
    if (!date.value) {
      errorEl.textContent = "Date is required.";
      errorEl.hidden = false;
      return;
    }
    successEl.textContent = "Claim submitted (fixture mock).";
    successEl.hidden = false;
  });

  approve.addEventListener("click", () => {
    successEl.textContent = "Claim approved (fixture mock).";
    successEl.hidden = false;
  });

  refresh();
})();
