#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Sinh `data/en_office_phrases.tsv` — bảng ngữ cảnh cụm từ/câu tiếng Anh văn phòng (P6).

Định dạng TSV khớp `en_phrases.tsv`: `context_lower \t predicted_phrase \t weight`
- context  : đuôi ngữ cảnh, lowercase, viết kiểu dấu cũ (kiểu "hòa") khớp
             `restyle(context, Old)` trong `core/src/post/predict.rs`.
- predicted: phần nối tiếp engine chèn sau ngữ cảnh (≤ 60 ký tự — MAX_TEXT/FFI).
- weight   : 1..=100; context dài hơn (cụ thể hơn) → weight cao hơn.

Nguồn sinh (không copy từ corpus bên ngoài — curated + template expansion):
  1. UPWARD     — báo cáo cấp trên: exec summary, escalation, risk/blocker,
                  milestone sign-off, RCA & mitigation update.
  2. DOWNWARD   — quản lý cấp dưới: delegation, alignment, review feedback,
                  deadline check-in, unblock, recognition.
  3. TECH       — kỹ thuật phần mềm: design/RCA/FCA, CI/CD, debug, perf,
                  deployment, API/integration.
  4. BUSINESS   — nghiệp vụ & dự án: status, sprint, KPI/ROI, vendor, scope,
                  action items, meeting follow-up.
  5. EVERYDAY   — keo kết hàng ngày: yêu cầu lịch sự, xác nhận, hẹn lịch,
                  collocation văn phòng.
  6. BILINGUAL  — key Việt-Eng code-switch ("mình sync lại" → "status ...")
                  để gợi ý tiếng Anh bắn ra giữa câu tiếng Việt.

Mỗi câu thật được chia ranh giới MỌI từ (key = w[0..i] → pred = w[i..]) giống
pattern `en_phrases.tsv` vòng 52. Chạy: `python scripts/gen_en_office_phrases.py`
(idempotent — ghi đè cùng nội dung; KHÔNG sửa tay file output).
"""

import itertools
import sys
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "data" / "en_office_phrases.tsv"

MAX_PRED = 60  # invariant `du_lieu_phrases_invariant` / MAX_TEXT=64
BASE_W = {  # weight cơ bản theo nhóm — cụ thể hoá +2/từ key, cap 100
    "upward": 82,
    "downward": 82,
    "tech": 84,
    "business": 84,
    "everyday": 76,
    "bilingual": 90,
}

# ---------------------------------------------------------------------------
# 1) Câu thật theo nhóm — mỗi template là danh sách segment; mỗi segment là
#    danh sách lựa chọn (mỗi lựa chọn = chuỗi hoàn chỉnh đọc được). Tích Descarte
#    trong mỗi template sinh mọi tổ hợp đọc tự nhiên.
# ---------------------------------------------------------------------------

SENTENCES = []  # (category, sentence)


def fam(category, *segments):
    for combo in itertools.product(*segments):
        SENTENCES.append((category, " ".join(s.strip() for s in combo).strip()))


# --- UPWARD: báo cáo lên lãnh đạo ------------------------------------------
fam(
    "upward",
    ["here is a quick status update on", "here is the latest update on"],
    ["the migration", "the rollout", "the audit", "the integration", "the pilot",
     "the vendor transition", "the security review", "the platform upgrade"],
)
fam(
    "upward",
    ["we are currently on track to"],
    ["deliver the first milestone", "close the phase one scope",
     "complete the migration", "ship the release candidate",
     "finalize the audit report", "hit the quarterly target",
     "wrap up the pilot phase", "close out the action items"],
)
fam(
    "upward",
    ["one key risk to highlight is"],
    ["the vendor delay", "the data migration gap", "the hiring shortfall",
     "the budget overrun", "the integration dependency",
     "the compliance review timeline", "the scope creep on workstream two"],
)
fam(
    "upward",
    ["escalating this blocker for your visibility"],
    ["before it impacts the deadline", "as it now needs an executive decision",
     "since the vendor has missed two checkpoints",
     "because the timeline can no longer absorb it"],
)
fam(
    "upward",
    ["please find the executive summary below"],
    ["with the key numbers highlighted", "covering this quarter only",
     "along with the recommended next steps"],
)
fam(
    "upward",
    ["please find the executive summary"],
    ["in the attached document", "in the linked deck", "on page one of the report",
     "at the end of this email"],
)
fam(
    "upward",
    ["we identified the root cause of"],
    ["the login failure", "the memory leak", "the timeout spikes",
     "the build failure", "the data drift", "the reporting mismatch",
     "the latency regression"],
    ["and shipped a fix", "and deployed a mitigation", "and added a guard",
     "and documented the workaround"],
)
fam(
    "upward",
    ["we identified the root cause"],
    ["and the fix has been verified on ci", "yesterday evening",
     "in the payment service", "during the incident review"],
)
fam(
    "upward",
    ["the migration is complete and"],
    ["all checks are green", "no data loss was detected",
     "the rollback plan stays on standby", "downstream teams have been notified"],
)
fam(
    "upward",
    ["milestone one is signed off and"],
    ["we are moving to phase two", "the budget stays within plan",
     "the team is already unblocked"],
)
fam(
    "upward",
    ["we are tracking slightly behind plan due to"],
    ["the delayed vendor delivery", "the extended testing cycle",
     "the scope added mid quarter", "the dependency on the api team"],
)
fam(
    "upward",
    ["we are tracking ahead of plan on"],
    ["the migration workstream", "the cost reduction program",
     "the migration of legacy records"],
)
fam(
    "upward",
    ["summary of this week"],
    ["for the leadership review", "in three bullets", "with risks and asks first"],
)
fam(
    "upward",
    ["our key achievement this week was"],
    ["closing the security gap", "landing the integration on time",
     "cutting the monthly cost by twenty percent",
     "unblocking the dependent teams"],
)
fam(
    "upward",
    ["we need a decision from you on"],
    ["the proposed scope cut", "the vendor shortlist", "the hiring plan",
     "the go live date", "the budget reallocation"],
)
fam(
    "upward",
    ["kindly confirm whether"],
    ["the proposed approach works for you", "you approve the revised timeline",
     "we should proceed with option two"],
)
fam(
    "upward",
    ["the quarterly numbers are"],
    ["in the attached dashboard", "on track against plan",
     "slightly under forecast this month"],
)
fam(
    "upward",
    ["regression risk after this change is"],
    ["low and fully covered by tests", "contained to the reporting module",
     "mitigated by the rollback plan"],
)
fam(
    "upward",
    ["we closed the incident with"],
    ["no customer impact", "a full postmortem published",
     "the fix verified in production"],
)
fam(
    "upward",
    ["next steps from our side are"],
    ["the dry run on friday", "the final security review",
     "the handover to the support team"],
)
fam(
    "upward",
    ["we would like your input on"],
    ["the prioritization between the two tracks",
     "the hiring sequence for next quarter",
     "the proposed pricing adjustment"],
)
fam(
    "upward",
    ["attaching the final report"],
    ["for your review", "for the board meeting", "with the appendix updated"],
)

# --- DOWNWARD: quản lý & điều phối ------------------------------------------
fam(
    "downward",
    ["could you please prioritize"],
    ["the login issue first", "this ticket over the others",
     "the customer report for today", "the hotfix for the payment flow"],
)
fam(
    "downward",
    ["lets align on the key deliverables for"],
    ["this sprint", "the next milestone", "the quarter", "the pilot phase"],
)
fam(
    "downward",
    ["please keep me posted on your progress"],
    ["by end of week", "twice a week", "as soon as the block is clear",
     "with a short daily note"],
)
fam(
    "downward",
    ["what support do you need to unblock"],
    ["the review", "the deployment", "the data access", "the testing phase"],
)
fam(
    "downward",
    ["great job on delivering"],
    ["the release on time", "the migration without downtime",
     "the demo to the client", "the audit findings under deadline"],
)
fam(
    "downward",
    ["thanks for turning around"],
    ["the review so quickly", "the report on short notice",
     "the fix over the weekend"],
)
fam(
    "downward",
    ["lets review the technical trade offs"],
    ["before we commit to the design", "in tomorrows sync",
     "between the two options"],
)
fam(
    "downward",
    ["please double check"],
    ["the numbers before we publish", "the rollback steps", "the test coverage",
     "the config on staging"],
)
fam(
    "downward",
    ["could you take over"],
    ["the client communication for this week",
     "the release notes for this build",
     "the on call rotation next week"],
)
fam(
    "downward",
    ["please share a short status by"],
    ["end of day", "friday morning", "the end of the sprint"],
)
fam(
    "downward",
    ["lets break this down into smaller tasks and"],
    ["assign owners today", "estimate them together", "track them on the board"],
)
fam(
    "downward",
    ["i noticed the deadline slipped and"],
    ["i want to understand the blockers", "lets adjust the plan together",
     "we should flag it to the sponsor"],
)
fam(
    "downward",
    ["your work on"],
    ["the migration was excellent", "the report was spot on",
     "the migration saved us days"],
)
fam(
    "downward",
    ["please walk me through"],
    ["the failing scenario", "the proposed architecture",
     "the numbers in the forecast"],
)
fam(
    "downward",
    ["lets schedule a quick sync to finalize"],
    ["the scope for phase two", "the demo script", "the launch checklist"],
)
fam(
    "downward",
    ["please prepare the demo environment by"],
    ["wednesday noon", "the end of this week", "the client visit"],
)
fam(
    "downward",
    ["i appreciate you picking up"],
    ["the extra review load", "the support handover", "the urgent fix"],
)
fam(
    "downward",
    ["before we merge"],
    ["lets run the full regression suite", "please get one more review",
     "double check the migration script"],
)
fam(
    "downward",
    ["lets make sure everyone is clear on"],
    ["the priorities for this week", "the definition of done",
     "the handoff expectations"],
)
fam(
    "downward",
    ["please document the decision in"],
    ["the design doc", "the runbook", "the onboarding notes"],
)

# --- TECH: kỹ thuật phần mềm -------------------------------------------------
fam(
    "tech",
    ["we identified the root cause of the failure as"],
    ["a stale cache entry", "a race condition in the worker",
     "a missing null check", "an expired token in the config",
     "a connection pool exhaustion", "a wrong feature flag default"],
)
fam(
    "tech",
    ["the fix has been verified on ci and"],
    ["is ready to merge", "will ship in the next build",
     "covered by a regression test"],
)
fam(
    "tech",
    ["deploying the hotfix to production"],
    ["in the next window", "after the smoke tests pass",
     "behind a feature flag", "with a staged rollout"],
)
fam(
    "tech",
    ["lets review the technical trade offs between"],
    ["the two storage options", "sql and the queue based design",
     "batch and streaming pipelines"],
)
fam(
    "tech",
    ["the api contract is finalized and"],
    ["the client teams can start integration", "the schema is frozen",
     "backward compatibility is preserved"],
)
fam(
    "tech",
    ["we are adding rate limiting to"],
    ["the public endpoints", "the webhook receiver", "the search api"],
)
fam(
    "tech",
    ["the performance test shows"],
    ["a forty percent improvement", "no regression at the ninety ninth percentile",
     "latency under fifty milliseconds at peak"],
)
fam(
    "tech",
    ["profiling points to"],
    ["a slow query in the reporting path", "an n plus one in the listing api",
     "excessive allocations in the parser"],
)
fam(
    "tech",
    ["we need to migrate the data before"],
    ["the schema change lands", "the cutover weekend",
     "the legacy service is retired"],
)
fam(
    "tech",
    ["the failing test is flaky due to"],
    ["a shared temp directory", "timing on the ci runner",
     "an unordered map iteration", "a network stub timeout"],
)
fam(
    "tech",
    ["infrastructure as code for the new stack"],
    ["is in review", "is merged and applied to staging",
     "will be provisioned this week"],
)
fam(
    "tech",
    ["observability for the new service includes"],
    ["structured logs and traces", "dashboards for latency and errors",
     "alerts on the golden signals"],
)
fam(
    "tech",
    ["the security scan flagged"],
    ["an outdated dependency", "a weak cipher in the config",
     "a missing header on the login route"],
)
fam(
    "tech",
    ["rollback plan for this release is"],
    ["documented in the runbook", "a single command via the pipeline",
     "tested on staging last week"],
)
fam(
    "tech",
    ["the database migration adds"],
    ["an index on the audit table", "a nullable column for the region",
     "a unique constraint on the external id"],
)
fam(
    "tech",
    ["unit tests cover the core parser and"],
    ["integration tests cover the api layer",
     "the fuzz target guards the config loader"],
)
fam(
    "tech",
    ["we are pinning the dependency versions to"],
    ["make the build reproducible", "avoid the surprise upgrade",
     "match the ci image"],
)
fam(
    "tech",
    ["the load test at twice peak traffic shows"],
    ["stable latency", "graceful degradation only", "no error spike"],
)
fam(
    "tech",
    ["code review feedback is minor and"],
    ["i can address it today", "mostly about naming", "already pushed as a patch"],
)
fam(
    "tech",
    ["the incident timeline shows the alert fired"],
    ["within two minutes", "before customer reports",
     "and the on call mitigated in fifteen minutes"],
)
fam(
    "tech",
    ["we are documenting the architecture decision in"],
    ["an adr in the repo", "the design doc", "the team wiki"],
)
fam(
    "tech",
    ["the backlog for the platform team is"],
    ["sorted by risk first", "groomed for the next sprint",
     "split into two epics"],
)

# --- BUSINESS: nghiệp vụ & dự án ---------------------------------------------
fam(
    "business",
    ["following up on our action items from"],
    ["yesterday meeting", "last weeks review", "the steering committee call"],
)
fam(
    "business",
    ["from a business perspective this change"],
    ["reduces cost per ticket", "improves time to value",
     "opens the door to the mid market segment"],
)
fam(
    "business",
    ["we need stakeholder sign off before"],
    ["we lock the scope", "the build phase starts",
     "we publish the revised timeline"],
)
fam(
    "business",
    ["lets schedule a quick sync to finalize"],
    ["the proposal", "the pricing model", "the rollout sequence"],
)
fam(
    "business",
    ["the project status is green with"],
    ["one risk on the vendor side", "no open blockers",
     "budget and schedule on plan"],
)
fam(
    "business",
    ["the client asked for"],
    ["a revised timeline", "a proof of value", "an executive summary",
     "a single point of contact"],
)
fam(
    "business",
    ["the roi projection for this initiative is"],
    ["positive within two quarters", "in the attached model",
     "based on the pilot numbers"],
)
fam(
    "business",
    ["scope change request received from"],
    ["the marketing team", "the operations lead", "the client sponsor"],
)
fam(
    "business",
    ["kpi movement this month shows"],
    ["churn down two points", "conversion up steadily",
     "support volume trending flat"],
)
fam(
    "business",
    ["the vendor response is due by"],
    ["end of month", "the contract review date", "next tuesday"],
)
fam(
    "business",
    ["procurement confirmed the new contract"],
    ["with the discount applied", "starting next quarter",
     "pending legal review"],
)
fam(
    "business",
    ["the sprint review covered"],
    ["the completed stories and the demo", "carried over work only",
     "the goal for the next sprint"],
)
fam(
    "business",
    ["please update the risk register with"],
    ["the new dependency", "the mitigation status", "the compliance finding"],
)
fam(
    "business",
    ["the budget variance this quarter is"],
    ["within tolerance", "driven by the cloud spend",
     "offset by the hiring delay"],
)
fam(
    "business",
    ["customer feedback from the pilot shows"],
    ["strong demand for the export feature",
     "satisfaction above the target",
     "two recurring friction points"],
)
fam(
    "business",
    ["the handover to the operations team includes"],
    ["the runbook and the faq", "the escalation contacts",
     "the support tier mapping"],
)
fam(
    "business",
    ["we aligned with the client on"],
    ["the acceptance criteria", "the pilot success metrics",
     "the reporting cadence"],
)
fam(
    "business",
    ["the forecast for next quarter assumes"],
    ["flat headcount", "the new pricing live in month one",
     "the current close rate"],
)
fam(
    "business",
    ["minutes of todays sync are"],
    ["in the shared doc", "summarized below", "circulated to the attendees"],
)
fam(
    "business",
    ["the audit is scheduled for"],
    ["the second week of next month", "the end of the quarter",
     "the week after the release"],
)

# --- EVERYDAY: keo kết hàng ngày --------------------------------------------
fam(
    "everyday",
    ["thanks for the quick turnaround on"],
    ["this request", "the review", "the document"],
)
fam(
    "everyday",
    ["sorry for the late reply"],
    ["i was out of office", "the week got away from me", "i was on leave"],
)
fam(
    "everyday",
    ["thanks for flagging this"],
    ["i will look into it today", "to me and the team",
     "before it reached the client"],
)
fam(
    "everyday",
    ["could we move the meeting to"],
    ["later today", "thursday morning", "next week"],
)
fam(
    "everyday",
    ["i have added it to my calendar for"],
    ["tomorrow morning", "friday afternoon", "next monday"],
)
fam(
    "everyday",
    ["confirming receipt of your message and"],
    ["i will follow up shortly", "i will revert by end of day",
     "the team is on it"],
)
fam(
    "everyday",
    ["as discussed"],
    ["i am sharing the updated timeline below",
     "please find the summary attached",
     "we will proceed with option two"],
)
fam(
    "everyday",
    ["quick reminder about"],
    ["the review at three", "the training session tomorrow",
     "the timesheet deadline"],
)
fam(
    "everyday",
    ["please let me know if"],
    ["anything is unclear", "the time does not work for you",
     "you need more context"],
)
fam(
    "everyday",
    ["hope you had a good break and"],
    ["welcome back", "the handover notes are ready", "i kept the list short"],
)
fam(
    "everyday",
    ["kindly review and approve"],
    ["the attached document", "the leave request", "the expense report"],
)
fam(
    "everyday",
    ["i am out of office"],
    ["until monday", "this friday", "for the rest of the week"],
)
fam(
    "everyday",
    ["feel free to reach out if"],
    ["you have any questions", "anything looks off",
     "you want to discuss before the call"],
)
fam(
    "everyday",
    ["looping in the team for"],
    ["visibility", "their input", "the next steps"],
)
fam(
    "everyday",
    ["noted with thanks and"],
    ["i will take it from here", "will update the ticket",
     "the deadline is safe"],
)

# --- Volume families — tích lớn cho độ phủ (mỗi combo đọc tự nhiên) ---------
fam(
    "upward",
    ["the team completed"],
    ["the code review", "the data migration", "the security assessment",
     "the user acceptance testing", "the capacity planning", "the cost analysis",
     "the vendor evaluation", "the risk assessment", "the architecture review",
     "the disaster recovery drill", "the quarterly close", "the documentation sweep"],
    ["this week", "ahead of schedule", "without any blockers",
     "and the results are attached", "and everything is archived"],
)
fam(
    "upward",
    ["we are seeing"],
    ["intermittent timeouts", "elevated error rates", "slow query performance",
     "unexpected token refreshes", "higher than usual queue depth"],
    ["in"],
    ["the reporting service", "the checkout flow", "the notification pipeline",
     "the data sync jobs", "the mobile api gateway"],
    ["and have"],
    ["opened an incident", "added extra monitoring", "a mitigation in place",
     "the on call engaged"],
)
fam(
    "upward",
    ["we closed"],
    ["twelve", "eighteen", "twenty", "nine", "fifteen"],
    ["issues this sprint including"],
    ["the flaky test suite", "the deployment gate", "the slow export",
     "the login edge case", "the reporting drift"],
)
fam(
    "upward",
    ["please review"],
    ["the revised proposal", "the migration checklist", "the draft runbook",
     "the pricing memo", "the project charter", "the incident postmortem"],
    ["and share"],
    ["your comments", "any concerns", "edits directly", "a short summary"],
    ["by"],
    ["friday", "end of day", "next tuesday", "the end of the week"],
)
fam(
    "upward",
    ["our ask for this week is"],
    ["a decision on the vendor shortlist", "one more reviewer for the audit",
     "budget approval for the extra seats", "feedback on the revised timeline",
     "a sponsor for the pilot readout", "alignment on the launch date",
     "support for the compliance exception", "your signature on the addendum",
     "a checkpoint with the client sponsor", "clarity on the hiring freeze"],
)
fam(
    "upward",
    ["the pilot with"],
    ["the sales team", "two enterprise accounts", "the ops crew",
     "the support org"],
    ["shows"],
    ["a measurable drop in handle time", "strong adoption from day one",
     "mixed results on the export speed", "clear demand for the mobile view"],
)
fam(
    "downward",
    ["please prepare"],
    ["the demo environment", "the release notes", "the onboarding deck",
     "the test summary", "the migration script", "the client agenda",
     "the retro board", "the weekly digest"],
    ["for"],
    ["the client visit", "the steering review", "fridays demo",
     "the handover call"],
)
fam(
    "downward",
    ["could you own"],
    ["the reporting migration", "the vendor follow up", "the test plan",
     "the documentation refresh", "the release checklist",
     "the on call handover"],
    ["until"],
    ["the milestone review", "end of quarter", "the client sign off",
     "the next sprint starts"],
)
fam(
    "downward",
    ["i would like you to shadow"],
    ["me", "the senior reviewer", "the release captain"],
    ["on"],
    ["the next incident call", "the client workshop", "the cutover run",
     "the estimation session"],
)
fam(
    "downward",
    ["lets celebrate"],
    ["the clean audit", "the on time release", "the hundredth deployment",
     "the pilot win", "the zero downtime cutover"],
    ["at"],
    ["fridays standup", "the team lunch", "our next all hands"],
)
fam(
    "downward",
    ["please run"],
    ["the full regression suite", "the smoke tests", "the load test",
     "the security scan", "the migration dry run", "the export benchmark"],
    ["before"],
    ["the release gate", "the demo to the client", "the cutover weekend",
     "you merge the branch"],
)
fam(
    "downward",
    ["make sure"],
    ["the rollback script is tested", "the alerts are silenced during the window",
     "the changelog is updated", "the staging data is fresh",
     "the reviewers are assigned", "the feature flag defaults to off",
     "the docs match the new flow", "the checklist is signed",
     "the client calendar holds the slot", "the demo account has test data"],
)
fam(
    "downward",
    ["if you hit a blocker on"],
    ["the data import", "the review queue", "the environment setup",
     "the client feedback", "the vendor api", "the test seeds"],
    ["ping"],
    ["me directly", "the platform channel", "the duty manager"],
)
fam(
    "tech",
    ["we added"],
    ["structured logging", "retry logic with backoff", "input validation",
     "a circuit breaker", "caching at the read path",
     "metrics on the queue depth"],
    ["to"],
    ["the payment service", "the webhook handler", "the background jobs",
     "the api gateway", "the auth flow"],
    ["so that"],
    ["failures surface faster", "retries no longer hammer the db",
     "bad requests are rejected early", "cascading failures are contained",
     "the tail latency drops"],
)
fam(
    "tech",
    ["the pipeline failed at"],
    ["the build step", "the unit tests", "the integration tests",
     "the security scan", "the artifact upload", "the deploy job"],
    ["due to"],
    ["a missing environment variable", "an expired credential",
     "a flaky network stub", "a dependency version conflict",
     "a disk space limit", "a timeout on the runner"],
)
fam(
    "tech",
    ["the api returns"],
    ["a four hundred", "a four oh one", "a four oh nine", "a five hundred"],
    ["when"],
    ["the payload misses a required field", "the token has expired",
     "the client retries too fast", "the upstream dependency is down"],
    ["we should"],
    ["document it in the reference", "add a retry hint to the response",
     "log the correlation id"],
)
fam(
    "tech",
    ["we are migrating"],
    ["the reporting jobs", "the static assets", "the audit logs"],
    ["from"],
    ["the legacy vm", "the on premise cluster", "the old bucket layout"],
    ["to"],
    ["the managed service", "the kubernetes pool", "the new storage tier"],
)
fam(
    "tech",
    ["the new build cuts"],
    ["cold start time", "peak latency", "binary size", "memory usage",
     "build duration"],
    ["by"],
    ["twenty percent", "a third", "half", "forty percent", "a wide margin"],
)
fam(
    "tech",
    ["tests for"],
    ["the parser", "the pricing engine", "the auth flow", "the export path",
     "the sync worker", "the cli commands"],
    ["cover"],
    ["the edge cases we hit last quarter", "unicode input",
     "concurrent writes", "malformed payloads"],
)
fam(
    "tech",
    ["the dashboard tracks"],
    ["p ninety nine latency", "error rate", "queue depth", "cache hit ratio"],
    ["and"],
    ["throughput", "active sessions", "retry count", "cost per request"],
    ["for"],
    ["the api tier", "the batch jobs", "the checkout flow", "the search stack"],
)
fam(
    "tech",
    ["we enabled"],
    ["the new scheduler", "graceful shutdown", "the replica autoscaling",
     "the compressed responses"],
    ["for"],
    ["ten percent", "a quarter", "half", "all"],
    ["of traffic"],
)
fam(
    "business",
    ["the meeting with"],
    ["the client sponsor", "the vendor", "the finance lead", "the auditors"],
    ["covered"],
    ["the revised scope", "the pricing model", "the rollout risks",
     "the open invoices", "the renewal terms"],
    ["and"],
    ["we agreed on next steps", "the action list is in the notes",
     "we need one more session", "the decision is documented"],
)
fam(
    "business",
    ["we are preparing"],
    ["the proposal", "the quarterly business review", "the onboarding pack",
     "the case study", "the training material", "the annual budget"],
    ["for"],
    ["the client leadership", "the board", "the new joiners",
     "the partner summit"],
)
fam(
    "business",
    ["the invoice from"],
    ["the cloud provider", "the design agency", "the staffing firm",
     "the data vendor"],
    ["is"],
    ["pending approval", "over budget by ten percent", "paid and closed",
     "disputed and under review"],
)
fam(
    "business",
    ["the renewal of"],
    ["the support contract", "the crm license", "the cloud commitment"],
    ["is due"],
    ["next month", "at the end of the quarter", "in sixty days"],
    ["and"],
    ["we should start negotiation early", "usage numbers support a downgrade",
     "the alternatives are priced in the deck"],
)
fam(
    "business",
    ["customer sentiment on"],
    ["the new dashboard", "the mobile app", "the onboarding flow",
     "the billing portal", "the search quality"],
    ["is"],
    ["trending positive", "mixed on performance", "a bright spot this quarter",
     "the top ask in interviews"],
)
fam(
    "business",
    ["the board asked for"],
    ["a cost breakdown by team", "the hiring plan for next year",
     "a risk snapshot of the roadmap", "the competitor pricing scan",
     "the pilot economics"],
    ["by"],
    ["the next meeting", "end of month", "friday", "the offsite"],
)
fam(
    "everyday",
    ["thanks again for"],
    ["the thorough review", "covering my shift", "the smooth handover",
     "the quick fix on friday", "organizing the offsite",
     "walking me through the setup", "the thoughtful feedback",
     "chasing the vendor for us", "the warm welcome", "the detailed notes"],
)
fam(
    "everyday",
    ["could you resend"],
    ["the deck from this morning", "the invite for thursday",
     "the last version of the doc", "the recording of the session",
     "the link to the board", "the invoice pdf", "the checklist template",
     "the seating plan"],
)
fam(
    "everyday",
    ["i will be at"],
    ["the partner summit", "the training day", "the client site",
     "the quarterly offsite"],
    ["so lets"],
    ["move our sync to thursday", "catch up after i am back",
     "keep the update async", "have someone else cover the demo"],
)
fam(
    "everyday",
    ["please add me to"],
    ["the release channel", "the audit thread", "the onboarding calendar",
     "the review rotation", "the client alias", "the backup list",
     "the design review invite", "the sprint board"],
)
fam(
    "everyday",
    ["it was great catching up at"],
    ["the offsite", "yesterdays lunch", "the conference",
     "the team dinner", "the all hands", "the workshop"],
)

# ---------------------------------------------------------------------------
# 2) Cụm standalone context → phrase (không qua câu dài) — collocation /
#    opener văn phòng dùng trực tiếp khi gõ opener rồi Tab.
# ---------------------------------------------------------------------------

STANDALONE = [
    # (key tail, prediction, weight)
    ("please find attached", "the signed agreement for your records", 88),
    ("please find the updated", "timeline for this workstream below", 86),
    ("please advise on", "the next steps for this request", 84),
    ("please be informed that", "the maintenance window moved to saturday", 80),
    ("please be advised that", "the deadline for submissions is friday", 80),
    ("as per our discussion", "i have updated the plan accordingly", 88),
    ("as per the policy", "the request needs one more approval", 82),
    ("as a follow up to", "our call this morning", 86),
    ("in the meantime", "we will prepare the fallback plan", 84),
    ("in the interest of time", "lets focus on the two blockers only", 82),
    ("at your earliest convenience", "please confirm the meeting slot", 82),
    ("for your reference", "the previous report is linked here", 80),
    ("for your information", "the release notes are published", 78),
    ("just a quick heads up that", "the deploy window changed", 84),
    ("quick heads up", "the staging refresh runs tonight", 84),
    ("a quick update on", "the outstanding action items", 86),
    ("one more thing before we close", "the invoice needs a second review", 78),
    ("to summarize the discussion", "we agreed on three action items", 86),
    ("to summarize", "we will pilot with two teams first", 84),
    ("to wrap up this thread", "the final decision is documented below", 80),
    ("happy to jump on a call if", "that is easier to align", 80),
    ("happy to help with", "the migration script anytime", 78),
    ("glad to hear that the issue", "is resolved on your side", 78),
    ("sorry for the confusion earlier", "the correct date is thursday", 78),
    ("my apologies for the oversight", "i have corrected the report", 78),
    ("thank you for your patience while", "we sorted out the access", 82),
    ("thank you for the detailed feedback", "it helped shape the final design", 82),
    ("looking forward to hearing your thoughts", "on the proposal", 88),
    ("looking forward to your feedback", "by friday if possible", 86),
    ("looking forward to working with", "the new team next quarter", 80),
    ("please treat this as", "the single source of truth going forward", 78),
    ("please do not hesitate to", "reach out with any questions", 84),
    ("please feel free to", "edit the doc directly", 80),
    ("kindly note that the deadline", "has moved to next wednesday", 82),
    ("kindly acknowledge receipt of", "this message and the attachment", 78),
    ("i would like to flag that", "the risk has grown since last week", 82),
    ("i would like to request", "an extension until friday", 82),
    ("i am writing to follow up on", "the open invoice from march", 84),
    ("i am writing to confirm that", "the review is booked for monday", 82),
    ("i am writing to escalate", "the recurring access issue", 84),
    ("we regret to inform you that", "the pilot window closed early", 74),
    ("we are pleased to share that", "the milestone shipped on time", 78),
    ("we are pleased to announce that", "the integration is now live", 78),
    ("on top of that", "the review cycle shortened by half", 74),
    ("by way of reminder", "the quiet period starts friday", 74),
    ("after further review", "we recommend option two", 80),
    ("after digging deeper into", "the logs we found the trigger", 78),
    ("based on the latest numbers", "we are adjusting the forecast", 80),
    ("based on your feedback", "we revised the rollout plan", 82),
    ("per the latest update from the vendor", "the fix lands next week", 78),
    ("during the review we found", "two minor gaps in the checklist", 78),
    ("while reviewing the report", "one anomaly stood out", 74),
    ("from the operations side", "everything is running as planned", 76),
    ("from a compliance standpoint", "the current setup is acceptable", 78),
    ("from a risk perspective", "the exposure stays within limits", 78),
    ("in terms of timeline", "we still hold the original date", 78),
    ("in terms of next steps", "the owner list is below", 78),
    ("with regard to your request", "the team has started on it", 78),
    ("with reference to the ticket", "the fix is queued for release", 78),
    ("apart from the items above", "no other risks were raised", 74),
    ("along the same lines", "the second module needs the same fix", 74),
    ("on a separate note", "the training videos are ready", 76),
    ("in parallel", "the legal team is drafting the addendum", 76),
    ("in preparation for the audit", "we are collecting the evidence", 78),
    ("ahead of the release", "please complete the checklist", 78),
    ("ahead of schedule on", "the integration workstream", 78),
    ("behind schedule on", "the reporting migration", 78),
    ("under review by", "the security team this week", 76),
    ("under discussion with the client", "the second phase scope", 76),
    ("pending approval from", "the finance lead", 76),
    ("pending confirmation from the vendor", "the maintenance slot", 76),
    ("subject to change until", "the contract is signed", 74),
    ("no action needed from you at", "this point in time", 76),
    ("action required by", "end of business friday", 80),
    ("action items from today", "are captured in the doc", 78),
    ("takeaways from the meeting", "in three short bullets", 76),
    ("next milestone is scheduled for", "the first week of june", 78),
    ("the scope for phase two covers", "reporting and exports", 78),
    ("the purpose of this email is to", "request your approval", 80),
    ("the goal for this sprint is", "the onboarding flow rewrite", 78),
    ("the plan for the coming week is", "finishing the dry run", 76),
    ("the outcome of the review was", "a go with two conditions", 78),
    ("the main takeaway is that", "the pilot met its target", 78),
    ("the open question is whether", "the vendor can hold the date", 78),
    ("the latest estimate puts delivery in", "the second week of next month", 78),
    ("the current status of the ticket is", "waiting on customer input", 78),
    ("the root cause was a misconfigured", "cache header on the edge", 80),
    ("the workaround for now is", "clearing the cache manually", 78),
    ("the long term fix will be", "part of the platform upgrade", 78),
    ("the short term mitigation is", "an extra review on the queue", 76),
    ("the impact assessment shows", "no downstream breakage", 78),
    ("the test results are attached", "for your review", 78),
    ("the meeting notes are shared", "with everyone on this thread", 76),
    ("the updated timeline is below", "please review by thursday", 78),
    ("the draft is ready for", "your first pass", 76),
    ("the final version is uploaded to", "the shared drive", 76),
    ("the retrospective highlighted", "one process fix to adopt", 74),
    ("the demo went well with", "positive feedback from the client", 78),
]

# ---------------------------------------------------------------------------
# 3) BILINGUAL — key Việt-Eng code-switch (kiểu dấu CŨ) → prediction trộn
#    Việt-Eng tự nhiên. Cho gợi ý tiếng Anh giữa câu tiếng Việt (VI mode).
# ---------------------------------------------------------------------------

BILINGUAL = [
    ("mình sync lại", "status với team sau họp nhé", 94),
    ("mình sẽ sync lại", "status với bạn vào chiều nay", 92),
    ("mình sẽ update", "timeline cho sprint sau vào buổi họp", 92),
    ("mình update", "timeline mới vào cuối tuần này", 92),
    ("anh check giúp em", "lỗi này trên bản build mới nhé", 92),
    ("anh check giúp mình", "cái config trên staging với", 92),
    ("em check giúp anh", "ticket này trên hệ thống test", 92),
    ("anh sẽ review", "pull request này trong chiều nay", 92),
    ("nhờ anh review giúp", "pull request này trước 5 giờ", 92),
    ("anh confirm giúp em", "deadline của ticket này với", 92),
    ("em confirm giúp mình", "blocker này đã xử lý xong chưa", 92),
    ("anh update em tiến độ", "của ticket với nhé", 90),
    ("mình deploy bản", "hotfix lên production tối nay", 92),
    ("bản hotfix đã được", "deploy lên production thành công", 92),
    ("đã deploy lên", "production và theo dõi dashboards", 92),
    ("fix đã được", "verify trên ci và sẵn sàng merge", 92),
    ("bug này đã được", "fix và có regression test bảo vệ", 92),
    ("mình merge pull request", "này sau khi review xong nhé", 92),
    ("pull request này đang", "chờ review của anh và một teammate", 92),
    ("mình cần stakeholder", "chốt scope trước khi bắt đầu build", 92),
    ("cần chốt scope với", "stakeholder trước cuối tuần này", 92),
    ("mình họp sync nhanh", "về timeline dự án lúc 3 giờ", 92),
    ("họp sync nhanh về", "tiến độ sprint lúc 10 giờ sáng", 92),
    ("mình gửi anh", "executive summary của quý này", 92),
    ("gửi anh executive summary", "kèm số liệu quan trọng nhé", 92),
    ("anh xem giúp em", "cái proposal đính kèm với nhé", 90),
    ("anh xem giúp mình", "bản demo script mới với", 90),
    ("em gửi anh báo cáo", "progress của tuần này", 92),
    ("báo cáo progress của", "tuần này anh xem giúp em nhé", 92),
    ("ticket này đang bị", "block do chờ data từ team kia", 92),
    ("việc này bị block bởi", "dependency từ team platform", 92),
    ("mình cần escalate", "vấn đề này lên ban giám đốc", 92),
    ("việc này cần escalate lên", "sếp vì vượt quá quyền quyết định", 92),
    ("anh nhắc mình", "deadline của sprint này với", 90),
    ("anh schedule giúp em", "cuộc họp với client vào thứ năm", 92),
    ("em schedule giúp anh", "buổi demo với khách hàng nhé", 92),
    ("mình sẽ onboard", "bạn mới vào team tuần sau", 90),
    ("mình sẽ escalate issue", "này cho team hỗ trợ nhé", 92),
    ("anh assign ticket này", "cho em được không", 90),
    ("mình assign việc này", "cho bạn trong sprint tới nhé", 90),
    ("anh cho em xin", "feedback về bản thiết kế mới", 92),
    ("anh feedback giúp em", "về thiết kế api mới nhé", 92),
    ("em xin anh một", "chance để trình bày demo này", 90),
    ("mình theo dõi", "metrics của hệ thống mỗi sáng", 90),
    ("mình sẽ theo dõi", "metrics này qua dashboard mới", 90),
    ("anh ưu tiên giúp", "ticket lỗi login trước nhé", 92),
    ("cần ưu tiên việc", "này lên đầu sprint hiện tại", 92),
    ("mình đã gửi yêu cầu", "access cho repo mới của team", 90),
    ("anh cấp giúp em", "access vào server staging với", 92),
    ("anh giúp em chạy", "lại pipeline trên ci nhé", 92),
    ("mình chạy lại", "pipeline cho đến khi mọi gate xanh", 90),
    ("anh em mình rà lại", "checklist trước khi release nhé", 90),
    ("trước khi release", "anh em mình rà lại checklist nhé", 90),
    ("bản release candidate", "đã sẵn sàng để kiểm thử", 92),
    ("mình cần rollback", "bản deploy vừa rồi ngay lập tức", 92),
    ("kế hoạch rollback cho", "bản release này nằm trong runbook", 92),
    ("mình sẽ escalate cho", "nhóm on call xử lý ngay", 92),
    ("anh em mình họp retro", "sprint này vào chiều thứ sáu", 90),
    ("mình đặt lịch demo", "với khách hàng vào sáng thứ ba", 92),
    ("mình đặt lịch sync", "với team dev vào sáng mai", 92),
    ("anh em mình review lại", "code của module này trước khi merge", 90),
    ("code của mình đã", "được review và test đầy đủ rồi", 90),
    ("mình sẽ handover", "phần tài liệu cho team support", 92),
    ("việc handover cho", "team support cần runbook đầy đủ", 92),
    ("mình xin phép hoãn", "deadline của ticket này một ngày", 92),
    ("deadline của việc này", "đã dời sang thứ năm tuần sau", 92),
    ("mình xin phép report", "lại tiến độ vào sáng mai nhé", 90),
    ("anh report lại giúp em", "số liệu của tháng này nhé", 92),
]


def sentence_rows(category, sentence):
    """Chia câu ranh giới MỌI từ: key = w[0..i] → pred = w[i..], i = 1..n-1."""
    words = sentence.split()
    out = []
    for i in range(1, len(words)):
        key = " ".join(words[:i])
        pred = " ".join(words[i:])
        if not key or not pred:
            continue
        if len(pred) > MAX_PRED or len(key) > MAX_PRED:
            continue
        base = BASE_W[category]
        weight = min(base + 2 * min(i, 6), 100)
        out.append((key, pred, weight))
    return out


def main():
    # Windows console mặc định cp1252 — print chữ Việt (\u1ead…) crash
    # UnicodeEncodeError dù file TSV đã ghi utf-8 xong. Ép stdout/stderr
    # UTF-8 trước mọi print (lỗi thật 2026-11-17 — ghi project-common-errors).
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8")
    rows = {}  # (key, pred) -> weight (giữ weight cao nhất khi trùng)

    def add(key, pred, weight):
        key = " ".join(key.split()).lower()
        pred = " ".join(pred.split()).lower()
        if not key or not pred or weight <= 0 or weight > 100:
            return
        if len(pred) > MAX_PRED or len(key) > MAX_PRED:
            return
        if not key.islower() or not pred.islower():
            return
        cur = rows.get((key, pred))
        if cur is None or weight > cur:
            rows[(key, pred)] = weight

    for category, sentence in SENTENCES:
        for k, p, w in sentence_rows(category, sentence):
            add(k, p, w)
    for k, p, w in STANDALONE:
        add(k, p, w)
    for k, p, w in BILINGUAL:
        add(k, p, w)

    # Bảo vệ dữ liệu: không dòng rác, đủ số lượng yêu cầu (> 10.000).
    if len(rows) < 10_000:
        print(f"FATAL: chỉ sinh được {len(rows)} hàng < 10.000 — mở rộng bảng",
              file=sys.stderr)
        sys.exit(1)

    lines = [
        "# en_office_phrases.tsv — ngữ cảnh cụm từ/câu tiếng Anh văn phòng (P6,",
        "# vòng 59): upward reporting / downward management / tech / business /",
        "# everyday glue / bilingual Viet-Eng code-switch. Sinh bởi",
        "# scripts/gen_en_office_phrases.py — KHÔNG sửa tay.",
        "# context_lower\tpredicted_phrase\tweight",
    ]
    for (key, pred), weight in sorted(rows.items()):
        lines.append(f"{key}\t{pred}\t{weight}")
    # newline="\n" — ép LF cả trên Windows (F-A2-1 audit vòng 59): mặc định
    # write_text dịch \n theo os.linesep → CRLF lệch chuẩn với 2 bảng anh em.
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")

    cats = {}
    for (key, _), _w in rows.items():
        cats[key.split()[0]] = cats.get(key.split()[0], 0) + 1
    print(f"OK: {OUT} — {len(rows)} hàng ({len(SENTENCES)} câu thật + "
          f"{len(STANDALONE)} standalone + {len(BILINGUAL)} bilingual)")
    top = sorted(cats.items(), key=lambda kv: -kv[1])[:8]
    print("  top opener keys:", ", ".join(f"{k}:{n}" for k, n in top))


if __name__ == "__main__":
    main()
