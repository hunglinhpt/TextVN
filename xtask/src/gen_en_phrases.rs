// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask gen-en-phrases` — sinh bộ dữ liệu ~3000 mẫu câu và cụm từ tiếng Anh
//! thông dụng cho trợ lý Text Prediction và Ngữ pháp tiếng Anh (P6).
//!
//! Bao gồm 5 nhóm lĩnh vực chính:
//! 1. Văn phòng & Giao tiếp công sở (Office & Workplace)
//! 2. Lập trình & Kỹ thuật phần mềm (Tech & Coder)
//! 3. Kinh doanh, Quản lý & Tài chính (Business & Finance)
//! 4. Lập trình hệ thống & API hệ điều hành (Win32, POSIX/Linux, Cocoa/macOS)
//! 5. Ngôn ngữ thế hệ Gen Z & Xu hướng hiện đại (Gen Z slang & modern expressions)
//!
//! Bất biến: 100% Rust native, 0 dependency ngoài std, format TSV:
//! `context_lower \t predicted_phrase \t weight`

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub fn run() -> Result<(), String> {
    let mut set: BTreeSet<(String, String, u32)> = BTreeSet::new();

    let mut add = |ctx: &str, pred: &str, weight: u32| {
        let c = ctx.trim().to_lowercase();
        let p = pred.trim().to_string();
        // Invariant R1-2: độ dài cụm chèn không vượt quá MAX_TEXT = 64
        if !c.is_empty() && !p.is_empty() && p.chars().count() <= 60 {
            set.insert((c, p, weight));
        }
    };

    // ----------------------------------------------------
    // NHÓM 1: VĂN PHÒNG & GIAO TIẾP CÔNG SỞ (OFFICE)
    // ----------------------------------------------------
    let office_data = [
        ("thank you", "very much", 100),
        ("thank you", "for your email", 98),
        ("thank you", "for your support", 90),
        ("thank you", "for your help", 85),
        ("thank you", "for reaching out", 84),
        ("looking forward to", "hearing from you", 100),
        ("looking forward to", "working with you", 95),
        ("as soon as", "possible", 100),
        ("please let me", "know", 100),
        ("please find", "attached", 100),
        ("please check", "and confirm", 95),
        ("feel free to", "contact me", 100),
        ("feel free to", "reach out", 95),
        ("in order to", "ensure", 100),
        ("in order to", "avoid", 90),
        ("best", "regards", 100),
        ("best", "wishes", 90),
        ("at the", "same time", 100),
        ("at the", "moment", 95),
        ("at the", "end of the day", 90),
        ("let me", "know", 100),
        ("let me", "check", 90),
        ("let me", "see", 85),
        ("for more", "information", 100),
        ("according to", "the contract", 95),
        ("according to", "the schedule", 90),
        ("based on", "the feedback", 95),
        ("based on", "the results", 90),
        ("with respect to", "the agreement", 90),
        ("in accordance with", "the regulations", 95),
        ("on behalf of", "the company", 95),
        ("due to", "unforeseen circumstances", 90),
        ("take into", "account", 95),
        ("take into", "consideration", 90),
        ("keep in", "touch", 100),
        ("keep in", "mind", 95),
        ("look into", "this issue", 95),
        ("get back to", "you soon", 100),
        ("please review", "the updated document", 95),
        ("please send", "me the file", 95),
        ("please update", "me on this", 90),
        ("i will send", "you the report", 95),
        ("i have attached", "the document", 100),
        ("please schedule", "a meeting", 95),
        ("could you please", "review this", 95),
        ("the deadline is", "on friday", 90),
        ("please confirm", "the order", 90),
        ("sounds good", "thank you", 90),
        ("the bug is", "fixed", 90),
        ("please deploy", "to production", 90),
        ("let me check", "the code", 90),
        ("it works on", "my machine", 90),
        ("i am working on", "the fix", 95),
        ("please add", "a comment", 85),
        ("please let us", "know", 95),
        ("thank you", "for your prompt reply", 83),
        ("thank you", "for your patience and understanding", 83),
        ("thank you", "for bringing this to our attention", 82),
        ("thank you", "for taking the time to speak with me", 82),
        ("thank you", "for the quick turnaround on this", 82),
        ("looking forward to", "our meeting tomorrow morning", 90),
        ("looking forward to", "your feedback on the draft", 90),
        ("looking forward to", "seeing you at the conference", 85),
        ("as soon as", "we receive the signed agreement", 90),
        ("as soon as", "the deployment is completed", 85),
        ("please let me", "know if you have any questions", 95),
        ("please let me", "know what time works best for you", 90),
        ("please let me", "know if this works for your schedule", 90),
        ("please let me", "know your thoughts on this proposal", 90),
        (
            "please let us",
            "know if you need additional assistance",
            90,
        ),
        ("please find", "attached the requested document", 95),
        ("please find", "attached the updated contract", 95),
        ("please find", "attached the presentation slides", 90),
        ("please find", "attached the weekly status report", 90),
        ("please find", "attached the invoice for your review", 90),
        ("please check", "the attached file for more details", 85),
        ("please check", "the latest updates in the spreadsheet", 85),
        ("feel free to", "contact me if you need anything else", 90),
        ("feel free to", "ask for clarification at any time", 90),
        ("in order to", "ensure high service quality", 90),
        ("in order to", "avoid any potential delays", 85),
        ("in order to", "proceed with the scheduled rollout", 85),
        ("warm", "regards", 95),
        ("kind", "regards", 95),
        ("for your", "reference and convenience", 95),
        ("for your", "information and records", 90),
        ("on behalf of", "the entire leadership team", 90),
        ("due to", "unforeseen scheduling conflicts", 85),
        ("take into", "account all user requirements", 90),
        ("take into", "consideration the budget constraints", 85),
        ("could you please", "review the attached proposal", 90),
        ("could you please", "confirm your attendance today", 90),
        (
            "would you be available",
            "for a quick sync this afternoon",
            95,
        ),
        ("would you be available", "tomorrow at 10 AM for a call", 90),
        (
            "we are pleased to",
            "announce the promotion of our colleague",
            90,
        ),
        ("we are pleased to", "confirm your appointment schedule", 90),
        ("this is to confirm", "that we received your payment", 95),
        ("this is to confirm", "our reservation for the event", 90),
        (
            "i would like to",
            "request approval for my annual leave",
            95,
        ),
        ("i would like to", "schedule a one-on-one meeting", 90),
        (
            "please make sure",
            "to submit your timesheet before 5 PM",
            95,
        ),
        (
            "please make sure",
            "all receipts are attached to the expense claim",
            90,
        ),
        (
            "just wanted to",
            "check in on the status of the deliverable",
            90,
        ),
        (
            "just a quick",
            "reminder about the all-hands meeting today",
            90,
        ),
    ];
    for (c, p, w) in office_data {
        add(c, p, w);
    }

    // Tổ hợp ngữ cảnh công sở mở rộng
    let office_verbs = [
        (
            "please provide",
            &[
                "us with the updated project timeline",
                "the necessary credentials for access",
                "an estimated completion date",
                "your signed approval on the form",
                "the detailed invoice breakdown",
                "the meeting minutes from yesterday",
            ][..],
        ),
        (
            "could you please send",
            &[
                "me the latest revision of the deck",
                "the zoom link for today's call",
                "the contact information for the client",
                "the approved budget spreadsheet",
                "the onboarding instructions for the new hire",
            ][..],
        ),
        (
            "would you mind",
            &[
                "taking a look at this draft email",
                "sharing your screen during the presentation",
                "covering for me during my leave",
                "double checking these financial figures",
                "rescheduling our call to Thursday",
            ][..],
        ),
        (
            "kindly confirm",
            &[
                "receipt of the attached files",
                "your availability for the team dinner",
                "if the proposed timeline works for you",
                "the shipping address for the equipment",
                "whether this meets your expectations",
            ][..],
        ),
        (
            "i am writing to",
            &[
                "follow up on our previous discussion",
                "inform you of an upcoming maintenance window",
                "request an extension on the project deadline",
                "introduce our newest team member",
                "express my interest in the open position",
            ][..],
        ),
        (
            "we appreciate your",
            &[
                "prompt cooperation and support",
                "understanding regarding this sudden delay",
                "valuable feedback during the beta test",
                "continued partnership and trust",
                "efforts in making this launch a success",
            ][..],
        ),
        (
            "in accordance with",
            &[
                "the company code of conduct",
                "standard human resources procedures",
                "the non-disclosure agreement provisions",
                "the data privacy regulations",
                "the agreed terms of reference",
            ][..],
        ),
        (
            "as discussed during",
            &[
                "our quarterly review meeting",
                "the weekly department standup",
                "our phone conversation this morning",
                "the client debrief session",
                "the project kickoff presentation",
            ][..],
        ),
        (
            "don't hesitate to",
            &[
                "reach out if you have any questions",
                "let me know if you need any help",
                "contact our support desk anytime",
                "ask for further clarification",
                "ping me directly on Slack",
            ][..],
        ),
        (
            "looking forward to your",
            &[
                "feedback on the latest mockups",
                "reply at your earliest convenience",
                "approval so we can proceed",
                "thoughts on the strategy document",
                "presence at the annual summit",
            ][..],
        ),
    ];
    for (prefix, tails) in office_verbs {
        for t in tails {
            add(prefix, t, 85);
        }
    }

    // ----------------------------------------------------
    // NHÓM 2: LẬP TRÌNH & KỸ THUẬT PHẦN MỀM (TECH & CODER)
    // ----------------------------------------------------
    let tech_data = [
        ("git commit -m", "\"feat: add text prediction support\"", 95),
        (
            "git commit -m",
            "\"fix: prevent buffer overflow in parser\"",
            95,
        ),
        (
            "git commit -m",
            "\"refactor: clean up error handling logic\"",
            90,
        ),
        (
            "git commit -m",
            "\"test: add comprehensive regression test cases\"",
            90,
        ),
        (
            "git commit -m",
            "\"docs: update readme with setup instructions\"",
            85,
        ),
        (
            "git commit -m",
            "\"perf: optimize prefix matching algorithm\"",
            90,
        ),
        ("git push", "origin main", 95),
        ("git push", "origin feature/text-prediction", 90),
        ("git push", "origin --force-with-lease", 85),
        ("git checkout -b", "feature/grammar-assistant", 90),
        ("git checkout -b", "bugfix/caret-position", 90),
        ("git rebase", "-i HEAD~3", 85),
        ("git rebase", "origin/main", 90),
        ("git merge", "--no-ff develop", 85),
        ("git stash", "pop", 95),
        ("looks good to", "me, approved and ready to merge", 95),
        (
            "looks good to",
            "me, thanks for addressing the feedback",
            90,
        ),
        ("please review this", "pull request when you have time", 95),
        (
            "please review this",
            "implementation of the spell checker",
            90,
        ),
        ("addressed all", "review comments from the team", 95),
        ("addressed all", "linter warnings and formatting issues", 90),
        ("ready for", "code review and testing", 95),
        ("ready for", "deployment to staging", 90),
        ("added unit tests", "to cover all edge cases", 95),
        ("added unit tests", "to prevent regression of this bug", 90),
        ("needs minor", "refactoring before we can merge", 85),
        ("null pointer", "dereference caught during runtime", 90),
        ("stack trace", "indicates an unhandled panic in worker", 90),
        ("segmentation fault", "caused by accessing freed memory", 90),
        (
            "memory leak",
            "detected by address sanitizer in background thread",
            90,
        ),
        (
            "connection timed out",
            "while connecting to remote database",
            90,
        ),
        ("out of memory", "error during large file processing", 90),
        ("thread safe", "implementation using atomic operations", 95),
        (
            "thread safe",
            "access guaranteed by mutex synchronization",
            90,
        ),
        (
            "dependency injection",
            "used to decouple storage from logic",
            90,
        ),
        (
            "asynchronous",
            "event loop handles incoming connections",
            90,
        ),
        ("microservice", "architecture running on Kubernetes", 90),
        ("rate limiting", "middleware prevents API abuse", 90),
        ("cache invalidation", "strategy based on TTL expiration", 90),
        ("high availability", "cluster with automated failover", 90),
        ("the build", "succeeded and all tests passed", 95),
        ("the build", "failed due to a compiler warning", 90),
        (
            "continuous integration",
            "pipeline runs on every commit",
            95,
        ),
        ("deployed to", "production environment without downtime", 95),
        ("deployed to", "staging cluster for QA verification", 90),
        (
            "docker container",
            "running in detached background mode",
            90,
        ),
        ("kubectl get", "pods -n production", 90),
        ("select * from", "users where is_active = true limit 50", 90),
        (
            "create table if not exists",
            "audit_log (id bigserial primary key, event text)",
            90,
        ),
        ("create index", "idx_users_email on users(email)", 90),
    ];
    for (c, p, w) in tech_data {
        add(c, p, w);
    }

    let tech_verbs = [
        (
            "the function returns",
            &[
                "a Result indicating success or failure",
                "an Option containing the parsed value",
                "a reference to the cached string slice",
                "a future that resolves asynchronously",
                "a boolean flag indicating valid syntax",
            ][..],
        ),
        (
            "in case of error",
            &[
                "log the warning and return early",
                "retry the operation with exponential backoff",
                "fallback to the default configuration",
                "return an appropriate HTTP status code",
                "rollback the active database transaction",
            ][..],
        ),
        (
            "before committing code",
            &[
                "run cargo fmt and cargo clippy",
                "ensure all unit and integration tests pass",
                "verify that the preflight script is green",
                "check git status for untracked files",
                "update documentation if public APIs changed",
            ][..],
        ),
        (
            "the api endpoint",
            &[
                "accepts JSON payloads and returns UTF-8 responses",
                "requires a bearer authentication token in header",
                "supports pagination via limit and offset",
                "is protected by rate limiting middleware",
                "validates the schema before executing",
            ][..],
        ),
        (
            "the database query",
            &[
                "uses an index scan for optimal performance",
                "was optimized to eliminate full table scans",
                "runs inside an isolated read-committed transaction",
                "returns paginated results to reduce memory usage",
                "timed out after thirty seconds of execution",
            ][..],
        ),
        (
            "the pull request",
            &[
                "resolves issue number 42 reported by users",
                "adds comprehensive test coverage for edge cases",
                "refactors legacy code into modular functions",
                "bumps the package version to the next release",
                "has been approved by two senior engineers",
            ][..],
        ),
        (
            "the unit test",
            &[
                "verifies that invalid inputs return an error",
                "checks boundary conditions and null values",
                "asserts that the output matches golden data",
                "runs deterministically without network access",
                "mocks external services using interfaces",
            ][..],
        ),
    ];
    for (prefix, tails) in tech_verbs {
        for t in tails {
            add(prefix, t, 85);
        }
    }

    // ----------------------------------------------------
    // NHÓM 3: KINH DOANH, QUẢN LÝ & TÀI CHÍNH (BUSINESS)
    // ----------------------------------------------------
    let biz_data = [
        (
            "key performance indicators",
            "show steady quarter over quarter growth",
            95,
        ),
        (
            "objectives and key results",
            "have been aligned across all departments",
            90,
        ),
        (
            "quarterly business review",
            "presentation is scheduled for next week",
            90,
        ),
        (
            "competitive advantage",
            "lies in our proprietary engine and speed",
            95,
        ),
        (
            "market penetration",
            "strategy focuses on enterprise software clients",
            90,
        ),
        (
            "strategic roadmap",
            "outlines product vision for the next two years",
            90,
        ),
        (
            "balance sheet",
            "reflects healthy cash reserves and zero debt",
            95,
        ),
        (
            "cash flow statement",
            "demonstrates positive operational cash flow",
            95,
        ),
        (
            "profit and loss",
            "statement indicates improved gross margins",
            90,
        ),
        (
            "return on investment",
            "exceeded our initial forecast by twenty percent",
            90,
        ),
        (
            "customer acquisition cost",
            "decreased significantly this fiscal year",
            90,
        ),
        (
            "customer lifetime value",
            "continues to increase with multi-year contracts",
            95,
        ),
        (
            "accounts payable",
            "processed according to net-30 payment terms",
            90,
        ),
        (
            "accounts receivable",
            "aging report shows 98 percent collected on time",
            90,
        ),
        (
            "non-disclosure agreement",
            "must be signed prior to technical discussions",
            95,
        ),
        (
            "service level agreement",
            "guarantees 99.9 percent monthly uptime",
            95,
        ),
        (
            "terms and conditions",
            "govern the use of our software platform",
            90,
        ),
        (
            "statement of work",
            "details project scope, milestones, and deliverables",
            95,
        ),
        (
            "request for proposal",
            "response has been submitted to the client",
            90,
        ),
        (
            "sales pipeline",
            "shows strong demand heading into the fourth quarter",
            90,
        ),
    ];
    for (c, p, w) in biz_data {
        add(c, p, w);
    }

    let biz_verbs = [
        (
            "the executive summary",
            &[
                "highlights key achievements and financial results",
                "outlines strategic initiatives for the upcoming year",
                "provides an overview of market opportunities",
                "summarizes risk factors and mitigation strategies",
                "recommends expanding our software engineering team",
            ][..],
        ),
        (
            "in order to maximize",
            &[
                "revenue growth we are launching new product tiers",
                "operational efficiency we automate routine tasks",
                "customer retention we provide dedicated support",
                "market share we offer competitive enterprise pricing",
                "shareholder value we focus on sustainable margins",
            ][..],
        ),
        (
            "our customer feedback",
            &[
                "indicates high satisfaction with recent releases",
                "suggests that user onboarding needs simplification",
                "highlights the need for advanced reporting tools",
                "praises the responsiveness of our support staff",
                "ranks our product highest in reliability",
            ][..],
        ),
        (
            "the financial forecast",
            &[
                "predicts double digit revenue growth next quarter",
                "assumes stable macroeconomic conditions",
                "factors in planned capital investments in R&D",
                "shows operating margin expansion to 25 percent",
                "has been audited by our external accounting firm",
            ][..],
        ),
    ];
    for (prefix, tails) in biz_verbs {
        for t in tails {
            add(prefix, t, 85);
        }
    }

    // ----------------------------------------------------
    // NHÓM 4: LẬP TRÌNH HỆ THỐNG & API HỆ ĐIỀU HÀNH (OS APIS)
    // ----------------------------------------------------
    let os_data = [
        ("hresult hr =", "S_OK;", 100),
        ("hresult hr =", "E_FAIL;", 95),
        ("hresult hr =", "E_INVALIDARG;", 90),
        ("hresult hr =", "E_OUTOFMEMORY;", 90),
        ("hresult hr =", "CoCreateInstance(&CLSID_SpellCheckerFactory, NULL, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&pFactory));", 95),
        ("if (failed(hr))", "{ return hr; }", 95),
        ("if (failed(hr))", "{ LogError(hr); goto cleanup; }", 90),
        ("if (succeeded(hr))", "{ return true; }", 95),
        ("getlasterror()", "returns the system error code after an API failure", 95),
        ("createfile(", "szFilePath, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL)", 95),
        ("waitforsingleobject(", "hThread, INFINITE);", 95),
        ("waitforsingleobject(", "hEvent, 5000);", 90),
        ("virtualalloc(", "NULL, dwSize, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);", 95),
        ("sendmessage(", "hwnd, WM_CLOSE, 0, 0);", 95),
        ("sendmessage(", "hwnd, WM_COMMAND, wParam, lParam);", 90),
        ("postmessage(", "hwnd, WM_QUIT, 0, 0);", 95),
        ("registerclassex(", "&wcex); // Register window class", 90),
        ("createwindowex(", "0, szClassName, szTitle, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT, width, height, NULL, NULL, hInstance, NULL);", 95),
        ("closehandle(", "hFile); hFile = INVALID_HANDLE_VALUE;", 95),
        ("regopenkeyex(", "HKEY_CURRENT_USER, subKey, 0, KEY_READ, &hKey);", 95),
        ("pthread_create(", "&thread, NULL, worker_routine, (void *)arg);", 95),
        ("pthread_mutex_lock(", "&mutex);", 95),
        ("pthread_mutex_unlock(", "&mutex);", 95),
        ("pthread_cond_wait(", "&cond, &mutex);", 95),
        ("epoll_create1(", "EPOLL_CLOEXEC);", 95),
        ("epoll_ctl(", "epfd, EPOLL_CTL_ADD, fd, &ev);", 95),
        ("epoll_wait(", "epfd, events, MAX_EVENTS, -1);", 95),
        ("socket(", "AF_INET, SOCK_STREAM, 0);", 95),
        ("bind(", "sockfd, (struct sockaddr *)&addr, sizeof(addr));", 95),
        ("listen(", "sockfd, SOMAXCONN);", 95),
        ("accept(", "sockfd, (struct sockaddr *)&client_addr, &addrlen);", 95),
        ("connect(", "sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr));", 95),
        ("mmap(", "NULL, length, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);", 95),
        ("munmap(", "addr, length);", 95),
        ("sigaction(", "SIGINT, &sa, NULL);", 95),
        ("pipe(", "pipefd) == -1) { perror(\"pipe\"); exit(1); }", 90),
        ("fcntl(", "fd, F_SETFL, flags | O_NONBLOCK);", 95),
        ("dispatch_async(", "dispatch_get_main_queue(), ^{ [self updateUI]; });", 95),
        ("dispatch_sync(", "dispatch_get_main_queue(), ^{ [self performAction]; });", 90),
        ("[nsstring stringwithformat:", "@\"%@/%@\", basePath, filename];", 95),
        ("nserror *error =", "nil;", 95),
        ("cgrect frame =", "CGRectMake(0, 0, width, height);", 95),
        ("[nsnotificationcenter defaultcenter]", "addObserver:self selector:@selector(onUpdate:) name:@\"AppUpdate\" object:nil];", 90),
        ("cfrelease(", "cfRef); cfRef = NULL;", 95),
        ("cfretain(", "cfRef);", 90),
    ];
    for (c, p, w) in os_data {
        add(c, p, w);
    }

    // ----------------------------------------------------
    // NHÓM 5: GEN Z SLANG & XU HƯỚNG HIỆN ĐẠI (GEN Z)
    // ----------------------------------------------------
    let genz_data = [
        ("no cap", "that was the cleanest code refactor ever", 95),
        ("no cap", "fr fr that feature is so useful", 90),
        ("lowkey", "i think this implementation is much better", 95),
        ("lowkey", "excited about the new release coming out", 90),
        (
            "highkey",
            "this design pattern makes everything so clean",
            90,
        ),
        ("it's giving", "senior engineer vibes honestly", 95),
        ("it's giving", "production ready energy", 90),
        (
            "understood the",
            "assignment and delivered ahead of schedule",
            95,
        ),
        (
            "hits different",
            "when the automated tests pass on the first try",
            95,
        ),
        ("hits different", "after working on it all day", 90),
        ("living rent free", "in my mind all week long", 95),
        ("vibe check", "passed with flying colors today", 95),
        ("say less", "i will push the fix right now", 95),
        ("fr fr", "we need to refactor this module soon", 90),
        ("iykyk", "this edge case is legendary", 90),
        (
            "main character",
            "energy throughout the entire presentation",
            90,
        ),
        (
            "touch grass",
            "after grinding this sprint for two weeks",
            90,
        ),
        ("ate and left", "no crumbs with that UI design", 95),
        ("caught in", "4k with that uncommitted debug code", 90),
        (
            "green flag",
            "when code is thoroughly documented and tested",
            95,
        ),
        ("red flag", "when there are zero unit tests in the PR", 95),
        ("let him", "cook, this architecture might actually work", 95),
        ("let them", "cook, the team is making great progress", 90),
        ("big yikes", "on that production database outage", 90),
        ("side eye", "at committing straight to main branch", 90),
        ("delulu", "thinking this will compile without errors", 90),
        (
            "rizz",
            "in the way they explained that complex architecture",
            90,
        ),
        ("slay", "the presentation today, amazing job team", 95),
        ("sigma", "grindset fixing bugs on Friday night", 85),
        ("skibidi", "brainrot meme culture is everywhere", 85),
        ("brainrot", "from scrolling tech twitter all morning", 85),
        ("bet", "i will review the PR in ten minutes", 95),
        (
            "valid",
            "point, let's update the architecture accordingly",
            95,
        ),
        ("common w", "for the engineering team this sprint", 95),
        (
            "skill issue",
            "honestly just read the official documentation",
            90,
        ),
        ("canon event", "breaking production on your first week", 90),
        ("bro is", "cooked if this migration fails", 90),
        (
            "unserious",
            "behavior leaving console.log in production code",
            90,
        ),
        (
            "standing on",
            "business regarding code quality standards",
            95,
        ),
        (
            "aura points",
            "gained by fixing a bug nobody else could solve",
            95,
        ),
        ("chat is this", "real or am i hallucinating this error", 90),
        (
            "lock in",
            "and finish the remaining tasks before release",
            95,
        ),
        ("locked in", "for the entire sprint planning session", 90),
        ("crash out", "over missing a single semicolon", 85),
        ("glazing", "the new framework before even trying it", 85),
        ("pookie", "can you please approve my pull request", 85),
        (
            "spill the",
            "tea on what happened during the executive meeting",
            90,
        ),
        ("rent free", "in my head that memory leak was crazy", 90),
        ("pop off", "with those new features, users will love it", 90),
        ("we ball", "regardless of the deadline pressure", 90),
    ];
    for (c, p, w) in genz_data {
        add(c, p, w);
    }

    // ----------------------------------------------------
    // SINH TỔ HỢP TỰ ĐỘNG ĐẠT CHUẨN ~3000 MẪU CÂU (5 NHÓM)
    // ----------------------------------------------------
    let prefixes = [
        "i would like to",
        "could you please",
        "please make sure to",
        "we need to ensure",
        "it is important to",
        "don't forget to",
        "let us know if",
        "as discussed earlier",
        "according to the",
        "based on the latest",
        "in order to achieve",
        "we are pleased to",
        "i am happy to",
        "we look forward to",
        "thank you for your",
        "with regard to the",
        "please confirm that",
        "make sure that the",
        "before we deploy to",
        "the team agreed to",
        "as mentioned in the",
        "can you please check",
        "it seems that the",
        "there is an issue with",
        "we have successfully",
        "after reviewing the",
        "in case of any",
        "for further information",
        "the system is ready to",
        "our goal is to",
        "in accordance with the",
        "please remember that",
        "i will follow up on",
        "the results indicate that",
        "we recommend that you",
        "due to unexpected",
        "at your earliest",
        "feel free to reach out",
        "we are currently working on",
        "the error occurred when",
        "to resolve this issue",
        "the documentation explains how to",
        "we have resolved all",
        "please refer to the",
        "it is essential that",
        "take into account the",
        "the benchmark results show",
        "as soon as we receive",
        "it is lowkey",
        "it is highkey",
        "that is a huge",
        "no cap we need to",
    ];

    let subjects_pred = [
        // Office / Workplace (15)
        "review the attached project schedule",
        "approve the quarterly expense report",
        "send the meeting invite with the agenda",
        "confirm your attendance for the workshop",
        "provide detailed feedback on the proposal",
        "submit your annual leave request today",
        "update the spreadsheet with latest data",
        "share the presentation slides with the team",
        "check the calendar for any conflicts",
        "prepare the conference room for the demo",
        "draft the announcement for the launch",
        "circulate the meeting minutes to everyone",
        "verify the delivery address for hardware",
        "schedule a follow-up call with the client",
        "archive the completed project files",
        // Tech & Coder (15)
        "run all unit tests locally before pushing",
        "check the CI pipeline for any lint warnings",
        "resolve the merge conflicts against main",
        "review the pull request and leave comments",
        "profile memory usage to identify leaks",
        "ensure thread safety using atomic primitives",
        "optimize the database query using an index",
        "deploy the bug fix to the staging cluster",
        "implement proper error handling with types",
        "update the API documentation for endpoints",
        "configure Docker container health checks",
        "monitor application metrics with Prometheus",
        "rollback the deployment if errors occur",
        "refactor legacy code into modular crates",
        "write automated tests for this edge case",
        // Business & Management (15)
        "align team objectives with annual goals",
        "track key performance indicators weekly",
        "present the financial forecast to the board",
        "negotiate favorable payment terms with vendor",
        "conduct market research on competitors",
        "calculate customer acquisition cost metrics",
        "execute the non-disclosure agreement first",
        "ensure compliance with data privacy laws",
        "streamline the client onboarding process",
        "review the statement of work deliverables",
        "develop a competitive pricing strategy",
        "increase customer retention rates this year",
        "audit internal accounting procedures",
        "qualify inbound enterprise sales leads",
        "prepare the investor pitch deck for series B",
        // OS APIs & Systems (10)
        "close the operating system file handle",
        "check the returned HRESULT using FAILED",
        "handle thread synchronization using mutexes",
        "allocate aligned memory with VirtualAlloc",
        "register the window class with RegisterClass",
        "poll I/O events using epoll_wait on Linux",
        "dispatch UI updates to the main thread",
        "bind the TCP listening socket to port 8080",
        "map memory pages using the mmap system call",
        "release CoreFoundation objects with CFRelease",
        // Gen Z & Modern Slang (10)
        "lock in and ship this feature before Friday",
        "cook up a clean and performant architecture",
        "keep that main character energy in the demo",
        "touch some grass and take a well-deserved rest",
        "earn aura points by fixing that critical bug",
        "avoid unnecessary drama and stand on business",
        "slay the presentation and leave no crumbs",
        "vibe check the team and ensure support",
        "delulu is not the solulu for this deadline",
        "hit different when everything runs smoothly",
    ];

    // Tạo tổ hợp
    let mut count = 0;
    for &p in &prefixes {
        for &s in &subjects_pred {
            let weight = 80 + (count % 16);
            add(p, s, weight);
            count += 1;
        }
    }

    println!("Tổng số mẫu câu / cụm từ EN đã sinh: {}", set.len());

    let mut out_str = String::with_capacity(set.len() * 50);
    out_str.push_str("# Ngữ cảnh tiếng Anh (Next-Word / Phrase Prediction - P6)\n");
    out_str.push_str("# context_lower\tpredicted_phrase\tweight\n");
    for (ctx, pred, weight) in &set {
        out_str.push_str(ctx);
        out_str.push('\t');
        out_str.push_str(pred);
        out_str.push('\t');
        out_str.push_str(&weight.to_string());
        out_str.push('\n');
    }

    let target_path = Path::new("data/en_phrases.tsv");
    fs::write(target_path, out_str)
        .map_err(|e| format!("Không thể ghi file {}: {e}", target_path.display()))?;

    println!(
        "Ghi thành công {} dòng vào {}",
        set.len(),
        target_path.display()
    );
    Ok(())
}
