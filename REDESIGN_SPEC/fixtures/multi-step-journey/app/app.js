(function () {
  const denied = new URLSearchParams(location.search).get("denied") === "1";
  const state = { name: "", email: "", ws: "", plan: "" };
  const panels = [null, document.getElementById("step-1"), document.getElementById("step-2"), document.getElementById("step-3")];
  const inds = [null, document.getElementById("step-ind-1"), document.getElementById("step-ind-2"), document.getElementById("step-ind-3")];

  function show(step) {
    for (let i = 1; i <= 3; i++) {
      panels[i].hidden = i !== step;
      inds[i].classList.toggle("active", i === step);
    }
  }

  function validEmail(v) {
    return typeof v === "string" && v.includes("@") && v.indexOf("@") > 0;
  }

  document.getElementById("next-1").addEventListener("click", () => {
    const err = document.getElementById("err-1");
    err.hidden = true;
    state.name = document.getElementById("full-name").value.trim();
    state.email = document.getElementById("email").value.trim();
    if (!state.name || !validEmail(state.email)) {
      err.textContent = "Full name and a valid work email are required.";
      err.hidden = false;
      return;
    }
    show(2);
  });

  document.getElementById("back-2").addEventListener("click", () => show(1));
  document.getElementById("next-2").addEventListener("click", () => {
    const err = document.getElementById("err-2");
    err.hidden = true;
    if (denied) {
      err.textContent = "You do not have permission to create a workspace.";
      err.hidden = false;
      return;
    }
    state.ws = document.getElementById("ws-name").value.trim();
    state.plan = document.getElementById("plan").value;
    if (!state.ws || !state.plan) {
      err.textContent = "Workspace name and plan are required.";
      err.hidden = false;
      return;
    }
    document.getElementById("sum-name").textContent = state.name;
    document.getElementById("sum-email").textContent = state.email;
    document.getElementById("sum-ws").textContent = state.ws;
    document.getElementById("sum-plan").textContent = state.plan;
    show(3);
  });

  document.getElementById("back-3").addEventListener("click", () => show(2));
  document.getElementById("confirm").addEventListener("click", () => {
    const success = document.getElementById("success");
    success.textContent = "Workspace ready.";
    success.hidden = false;
  });

  show(1);
})();
