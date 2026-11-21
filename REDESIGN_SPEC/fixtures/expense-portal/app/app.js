// Expense portal fixture: a list, a detail page with an actions menu, and a four-step new-claim
// wizard. Rules and messages are listed in MANIFEST.md; the weak spots are deliberate.
const role = new URLSearchParams(location.search).get('role') === 'viewer' ? 'viewer' : 'employee';
const claims = [
  { id: 'C-1041', what: 'Train to Lyon', amount: '84.50', status: 'Approved' },
  { id: 'C-1042', what: 'Client dinner', amount: '212.00', status: 'Pending' },
  { id: 'C-1043', what: 'Monitor stand', amount: '39.99', status: 'Rejected' },
];
let draft = { what: '', amount: '', category: '', date: '', approver: '' };
let errors = [];
const view = document.getElementById('view');
document.getElementById('who').textContent = 'Signed in as ' + (role === 'viewer' ? 'Viewer' : 'Employee');
const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const errorBox = () => errors.length ? `<div class="errors" role="alert"><b>Please fix the following:</b><ul>${errors.map(e => `<li>${esc(e)}</li>`).join('')}</ul></div>` : '';
const intro = text => `<p class="intro">${text}</p>`.repeat(1);
const LONG = 'Please read the expense policy before filling in this step. Claims must relate to business activity, be submitted within ninety days of the expense and be supported by a receipt where the policy requires one. Amounts are entered in euros. Claims above the approval limit need an approver code from your manager. Incomplete claims are returned. ';

function render() {
  const hash = location.hash || '#/claims';
  if (hash === '#/claims') return list();
  if (hash.startsWith('#/claims/')) return detail(hash.slice(9));
  if (hash.startsWith('#/new')) return wizard(Number(hash.split('/')[2] || 1));
  view.innerHTML = '<p>Not found.</p>';
}

function list() {
  view.innerHTML = `<h1>My claims</h1>
    <table class="claims"><thead><tr><th>Claim</th><th>What</th><th>Amount (EUR)</th><th>Status</th><th></th></tr></thead><tbody>
    ${claims.map(c => `<tr><td>${c.id}</td><td>${esc(c.what)}</td><td>${c.amount}</td><td class="status">${c.status}</td><td><a href="#/claims/${c.id}">Open ${c.id}</a></td></tr>`).join('')}
    </tbody></table>`;
}

function detail(id) {
  const c = claims.find(x => x.id === id);
  if (!c) { view.innerHTML = '<p>Not found.</p>'; return; }
  view.innerHTML = `<h1>Claim ${c.id}</h1><p>${esc(c.what)} · ${c.amount} EUR</p><p class="status">Status: ${c.status}</p>
    <div class="more"><button id="more" aria-haspopup="true" aria-expanded="false">More</button><div id="menu" class="menu" hidden>
    <button id="withdraw">Withdraw claim</button><button id="copy">Copy as new</button></div></div><p id="msg" role="status"></p>`;
  const more = document.getElementById('more'), menu = document.getElementById('menu');
  more.onclick = () => { menu.hidden = !menu.hidden; more.setAttribute('aria-expanded', String(!menu.hidden)); };
  document.getElementById('withdraw').onclick = () => {
    if (role === 'viewer') { document.getElementById('msg').textContent = 'Viewers cannot withdraw claims.'; return; }
    if (c.status !== 'Pending') { document.getElementById('msg').textContent = 'Only pending claims can be withdrawn.'; return; }
    c.status = 'Withdrawn';
    document.getElementById('msg').textContent = `Claim ${c.id} withdrawn.`;
  };
  document.getElementById('copy').onclick = () => { draft = { what: c.what, amount: c.amount, category: '', date: '', approver: '' }; location.hash = '#/new/1'; };
}

function field(label, key, type = 'text', extra = '') {
  return `<label>${label}<input id="f-${key}" type="${type}" value="${esc(draft[key])}" ${extra}></label>`;
}

function wizard(step) {
  const steps = ['Details', 'Category and date', 'Approval', 'Confirm'];
  const head = `<h1>New claim</h1><p>Step ${step} of 4: ${steps[step - 1]}</p>${errorBox()}`;
  if (step === 1) {
    view.innerHTML = head + intro(LONG + LONG) + field('What was it for', 'what') + field('Amount (EUR)', 'amount') + '<button id="next">Next</button>';
  } else if (step === 2) {
    view.innerHTML = head + intro(LONG) + `<label>Category<select id="f-category"><option value="">Choose…</option>${['Travel', 'Meals', 'Equipment'].map(o => `<option ${draft.category === o ? 'selected' : ''}>${o}</option>`).join('')}</select></label>` + field('Date of expense', 'date', 'date') + '<button id="back">Back</button> <button id="next">Next</button>';
  } else if (step === 3) {
    const needs = Number(draft.amount) > 300;
    view.innerHTML = head + intro(LONG) + (needs ? field('Approver code', 'approver') : '<p>No approver needed for claims up to 300 EUR.</p>') + '<button id="back">Back</button> <button id="next">Next</button>';
  } else {
    view.innerHTML = head + `<p>You are about to submit a claim for ${esc(draft.what)} (${esc(draft.amount)} EUR, ${esc(draft.category)}, ${esc(draft.date)}).</p><p>Are you sure?</p>` +
      '<button id="back">Back</button> <button id="submit">Yes, submit claim</button><p id="msg" role="status"></p>';
  }
  const read = () => { for (const k of Object.keys(draft)) { const el = document.getElementById('f-' + k); if (el) draft[k] = el.value; } };
  const back = document.getElementById('back');
  if (back) back.onclick = () => { read(); errors = []; location.hash = '#/new/' + (step - 1); };
  const next = document.getElementById('next');
  if (next) next.onclick = () => {
    read(); errors = [];
    if (step === 1) { if (!draft.what.trim()) errors.push('What was it for is required.'); if (!(Number(draft.amount) > 0)) errors.push('Amount must be greater than 0.'); }
    if (step === 2) { if (!draft.category) errors.push('Choose a category.'); if (!draft.date) errors.push('Enter the date of the expense.'); }
    if (step === 3 && Number(draft.amount) > 300 && !draft.approver.trim()) errors.push('Claims over 300 EUR need an approver code.');
    if (errors.length) { render(); return; }
    location.hash = '#/new/' + (step + 1);
  };
  const submit = document.getElementById('submit');
  if (submit) submit.onclick = () => {
    if (role === 'viewer') { document.getElementById('msg').textContent = 'Viewers cannot submit claims.'; return; }
    claims.push({ id: 'C-' + (1041 + claims.length), what: draft.what, amount: draft.amount, status: 'Pending' });
    view.innerHTML = '<h1>New claim</h1><p class="done" role="status">Claim submitted for approval.</p><a href="#/claims">Back to my claims</a>';
    draft = { what: '', amount: '', category: '', date: '', approver: '' };
  };
}

window.addEventListener('hashchange', render);
render();
