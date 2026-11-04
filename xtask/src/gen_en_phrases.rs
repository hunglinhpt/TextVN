// SPDX-License-Identifier: GPL-3.0-or-later
//! `cargo xtask gen-en-phrases` — sinh bộ dữ liệu mẫu câu và cụm từ tiếng Anh
//! cho Text Prediction (P6) — **vòng 52 (2026-11-04): mở rộng theo cấu trúc
//! mới của `gen_vn_phrases`**:
//! 1. **Câu thật đầy đủ** (office/email/business/customer service/life): chia
//!    tại MỌI ranh giới từ — mọi dòng đều đúng ngữ nghĩa.
//! 2. **English idioms** (chỉ mục thật): chia tại mọi điểm.
//! 3. **Cặp curated + ma trận ngữ nghĩa-controlled** (subject×predicate,
//!    verb×object, deadline frames) — không cross-product vô nghĩa.
//!
//! Bất biến: 100% Rust native, 0 dependency ngoài std, format TSV
//! `context_lower \t predicted_phrase \t weight`, prediction ≤ 60 ký tự (R1-2).

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
            // 2 nguồn trùng (key,pred) khác weight — giữ MAX (audit vòng 52).
            let existing = set
                .iter()
                .find(|(k, pr, _)| k == &c && pr == &p)
                .map(|(_, _, w)| *w);
            match existing {
                Some(w) if w >= weight => {}
                Some(_) => {
                    let dupes: Vec<_> = set
                        .iter()
                        .filter(|(k, pr, _)| k == &c && pr == &p)
                        .cloned()
                        .collect();
                    for d in dupes {
                        set.remove(&d);
                    }
                    set.insert((c, p, weight));
                }
                None => {
                    set.insert((c, p, weight));
                }
            }
        }
    };

    // ------------------------------------------------------------------
    // 1. CÂU THẬT ĐẦY ĐỦ — chia tại mọi ranh giới từ.
    // ------------------------------------------------------------------
    let sentences: &[&str] = &[
        // ---- Email openers / formal (14) ----
        "I hope this email finds you well",
        "Thank you for your email and for your patience",
        "I am writing to follow up on our meeting last week",
        "I am writing to confirm the details of our upcoming call",
        "Thank you for reaching out regarding the new proposal",
        "Please find attached the report you requested",
        "Please find the updated version of the document attached",
        "Further to our conversation I am pleased to confirm the schedule",
        "As discussed I have attached the revised timeline for the project",
        "I wanted to let you know that the issue has been resolved",
        "I wanted to quickly follow up on the status of the invoice",
        "Just a quick reminder that the deadline is this Friday",
        "Just checking in to see if you had a chance to review",
        "Thanks for your quick response to my last email",
        // ---- Email bodies / requests (16) ----
        "Could you please send me the final version by end of day",
        "Could you please let me know your availability for next week",
        "Please let me know if you need any additional information",
        "Please let me know if the schedule works for you",
        "Please review the attached document and share your feedback",
        "Kindly confirm your attendance by Thursday afternoon",
        "Kindly review the terms and conditions before signing",
        "We would appreciate your feedback on the first draft",
        "We would like to request your approval for the budget plan",
        "Would you mind taking a quick look at the numbers below",
        "Would it be possible to reschedule our meeting to next Tuesday",
        "I have attached the invoice for your reference",
        "I have forwarded your request to the relevant department",
        "I will get back to you as soon as I have an update",
        "I will keep you posted on any further developments",
        "Let me know if you have any questions about the proposal",
        // ---- Email closers (10) ----
        "Thank you in advance for your support",
        "Thank you again for your time and consideration",
        "Looking forward to hearing from you soon",
        "Looking forward to working with you on this project",
        "Please do not hesitate to contact me if you need anything",
        "Should you have any questions please feel free to reach out",
        "Best regards and have a great week ahead",
        "I appreciate your help with this matter",
        "Thank you for your understanding and cooperation",
        "Have a wonderful day and talk to you soon",
        // ---- Meetings / project management (14) ----
        "The meeting is scheduled for ten o'clock on Monday morning",
        "The meeting has been moved to next Wednesday due to a scheduling conflict",
        "Please review the agenda before our call tomorrow",
        "The minutes of the meeting will be shared by end of day",
        "We need to revisit the timeline for the second phase of the project",
        "The project is currently on track and within budget",
        "The project is slightly behind schedule due to resource constraints",
        "We should prioritize the most urgent tasks first",
        "Let's schedule a quick sync to align on the next steps",
        "Let's touch base early next week to review the progress",
        "Everyone should come prepared with their status updates",
        "The client has requested some changes to the original scope",
        "We have reached an agreement on the key deliverables",
        "The next milestone is due at the end of this month",
        // ---- Accounting / finance (12) ----
        "The invoice has been processed and payment will be issued this week",
        "Please review the attached statement and confirm the balance",
        "The payment has been transferred to your bank account",
        "We have not yet received the payment for invoice one zero two",
        "The budget for next quarter is currently being finalized",
        "The report shows a ten percent increase in operating costs",
        "All expenses must be submitted with proper receipts attached",
        "The financial statements have been audited and approved",
        "Kindly reconcile the outstanding balance before month end",
        "The refund has been issued to your original payment method",
        "Please send the tax documents before the filing deadline",
        "The accounts payable team will process the request shortly",
        // ---- Human resources (10) ----
        "Your leave request has been approved by your manager",
        "The interview is scheduled for two o'clock on Thursday",
        "Please submit your timesheet by the end of the week",
        "The new employee orientation will take place next Monday",
        "Your performance review is scheduled for the end of this month",
        "We are currently hiring for two positions in the sales team",
        "Please update your emergency contact information in the system",
        "The benefits package includes health insurance and annual bonus",
        "The contract will be renewed for another twelve months",
        "Please complete the training module before the deadline",
        // ---- Customer service (14) ----
        "Thank you for contacting our support team",
        "We apologize for the inconvenience caused by this delay",
        "We are currently looking into this issue for you",
        "Your request has been received and is being processed",
        "Our team will contact you within twenty four hours",
        "The issue has been resolved and the service is back online",
        "You can track your order status on our website",
        "Your order has been shipped and will arrive in two days",
        "We have issued a full refund to your account",
        "Please do not hesitate to contact us if you need further assistance",
        "We value your feedback and appreciate your patience",
        "The warranty covers all manufacturing defects for two years",
        "Our technical team is investigating the root cause of the problem",
        "We would like to offer you a discount on your next purchase",
        // ---- Sales / business (10) ----
        "The quotation is valid for thirty days from the date of issue",
        "We are pleased to offer you a special discount this month",
        "Our team will contact you to schedule a product demonstration",
        "The contract includes a twelve month maintenance package",
        "We look forward to a long and successful partnership",
        "Please let us know your preferred delivery schedule",
        "The minimum order quantity for this product is fifty units",
        "The price list has been updated for the new fiscal year",
        "We can offer flexible payment terms for large orders",
        "The proposal includes a detailed breakdown of all costs",
        // ---- Tech / IT (10) ----
        "The system is scheduled for maintenance tonight at midnight",
        "Please save your work before the maintenance window begins",
        "The new version has been deployed to all production servers",
        "The server response time has returned to normal levels",
        "Please clear your browser cache and try logging in again",
        "The login issue has been identified and is being fixed",
        "Please reset your password using the link in the email",
        "The backup completed successfully last night",
        "The data will be encrypted both in transit and at rest",
        "Please report any unusual behavior to the security team",
        // ---- Small talk / life (14) ----
        "How was your weekend with the family",
        "How have you been since we last met",
        "It was great catching up with you last week",
        "Thanks again for helping me out with the report",
        "Let me know when you are free for a quick coffee",
        "Are you joining us for lunch at the new place",
        "The weather has been really nice this week",
        "Traffic was terrible this morning on the way to work",
        "I will be working from home tomorrow morning",
        "I am taking a few days off next week for a family trip",
        "Happy birthday and have a fantastic celebration",
        "Congratulations on the promotion and well deserved",
        "Get well soon and take all the time you need",
        "Have a safe trip and enjoy the holiday",
        // ---- Scheduling / calendar (8) ----
        "I need to reschedule our meeting to Thursday afternoon",
        "Please send me a calendar invite for the review session",
        "The appointment has been confirmed for ten thirty tomorrow",
        "I am available any time between two and five o'clock",
        "The deadline for the final submission is this Friday",
        "Please block my calendar for the training session",
        "The delivery is scheduled for Monday morning",
        "Can we move the call to earlier in the day",
        "The landlord agreed to renew the lease for another year",
        "The apartment has two bedrooms, a living room and a balcony",
        "We need to review the rental agreement before signing the extension",
        "The monthly fee covers utilities and one parking spot",
        "The ownership transfer will be notarized this week",
        "The property value in this area rises about five percent a year",
        "Please pay the deposit to hold the apartment until the end of the month",
        "We need a copy of the ownership certificate from both parties",
        "The new complex has a school and a supermarket on the ground floor",
        "Please schedule a viewing for the house listing this weekend",
        // ---- Banking / insurance (14) ----
        "I would like to open a savings account with a six month term",
        "The interest rate for this term is higher than the previous one",
        "There is an unusual transaction on my statement that needs investigation",
        "Please confirm the one time password before completing the transfer",
        "My credit card was locked after three wrong PIN attempts",
        "The home loan application needs the employment contract and salary statements",
        "The bank will disburse the loan within three business days after approval",
        "The annual card fee is waived for priority customers",
        "My life insurance premium is due at the end of this month",
        "The claims process usually takes about fifteen business days",
        "Please submit the claim form together with the receipts",
        "The payout amount is calculated according to the policy terms",
        "We received your application and will send the policy card this week",
        "The beneficiary needs to be present when signing the release documents",
        // ---- Logistics / import-export (14) ----
        "The customs declaration for this shipment was cleared this morning",
        "The shipment is expected to arrive at the port next Thursday",
        "We need to supplement the import documents for shipment forty two",
        "The import duty is calculated based on the HS code of the goods",
        "The goods will be moved to the central warehouse right after customs clearance",
        "The end of month inventory is slightly higher due to year end orders",
        "Please recount the quantity of goods in these four containers",
        "The shipping fee includes loading charges and one day of storage",
        "The carrier commits to delivering on schedule within two days",
        "The damaged package needs an inspection report with the carrier on site",
        "We will send the detailed delivery schedule for the whole month",
        "The freight cost increased with fuel prices this quarter",
        "Please ask the seller to send the bill of lading and commercial invoice",
        "The defective goods will be returned to the supplier with an outbound note",
        // ---- Manufacturing / engineering (12) ----
        "The production line will stop for maintenance this Sunday",
        "The factory output this quarter is five percent above plan",
        "Please restock raw materials for next week's production run",
        "The replacement parts will arrive at the workshop within two days",
        "Safety equipment is mandatory inside the production area",
        "The technical issue was fixed and the line is running normally again",
        "The product technical standard is specified in the appendix",
        "The defect rate of this batch is below the allowed threshold",
        "The technical file needs updating with the latest shop drawings",
        "The design capacity of the plant is fifty thousand units a year",
        "The quality control process consists of three main inspection steps",
        "The design drawings were approved and sent to the construction contractor",
        // ---- Tendering / contracts (12) ----
        "The bid documents must be submitted before five pm on the fifteenth",
        "This tender requires a bid guarantee issued by a bank",
        "We submitted bids for three packages this quarter",
        "The selection result will be announced next week",
        "The payment terms include three installments based on progress",
        "The memorandum signed yesterday takes effect from today",
        "Please review the warranty clause before signing",
        "The contract value is adjusted according to the new unit price list",
        "We commit to performing the full scope of work in the contract",
        "The contract liquidation will be done after final acceptance",
        "The buyer requests a one week extension for receiving the goods",
        "Any contract change must be recorded in a signed appendix",
        // ---- Media / content (10) ----
        "The product launch article has been edited and published",
        "The press release will go out on all official channels at once",
        "We need a detailed script for the introduction video",
        "The communication plan for next quarter focuses on two main channels",
        "The campaign reached ten thousand interactions in the first three days",
        "Please review the content before posting to the official page",
        "The press interview will take place on Wednesday afternoon",
        "The brand identity has been updated across all materials",
        "The promotional video is being finalized and will air next month",
        "Let's measure each channel's performance to reallocate the ad budget",
        // ---- Research / higher education (10) ----
        "Our research project has been approved with full funding",
        "The literature review will be presented at the academic conference in May",
        "The survey shows seventy percent of users are satisfied",
        "The questionnaire was sent to two hundred customers this week",
        "The research method must clearly describe how data was collected",
        "My thesis defense is scheduled for early June",
        "The references need updating with the new citation style",
        "The experiment results match the group's initial hypothesis",
        "The scholarship is awarded based on academic results and activities",
        "The business administration program is accepting graduate students",
        // ---- Construction (8) ----
        "The project finished the foundation and is ready for the frame",
        "The site supervisor asked for the as built drawings this month",
        "Construction materials are inspected before entering the site",
        "Construction progress is at seventy percent of the contract volume",
        "Site safety is the number one priority for every team member",
        "Final acceptance is expected at the end of next month after finishing",
        "The shop drawings were revised according to the supervisor's feedback",
        "The contractor commits to handover on the contractual deadline",
        // ---- Travel / hotels (10) ----
        "I would like to book a double room for two nights from Friday to Sunday",
        "My flight arrives at the airport at nine on Friday evening",
        "The round trip ticket has been issued and sent to your email",
        "Check in starts at two pm and check out is before noon",
        "Please book one more single room for my colleague",
        "My checked baggage was lost and I need to work with the agent",
        "The three day tour includes lunch and entrance tickets",
        "The detailed itinerary will be sent one week before departure",
        "The hotel offers airport shuttle service for a small fee",
        "We cancelled the booking due to a schedule conflict, sorry for the trouble",
        // ---- Healthcare (10) ----
        "I would like to take a day off to take my father to the doctor",
        "The lab results will be ready within two business days",
        "The doctor scheduled a follow up visit in two weeks",
        "This prescription needs to be filled at a pharmacy with a licensed pharmacist",
        "Health insurance covers seventy percent of the treatment cost",
        "The clinic receives patients from morning until seven in the evening",
        "Please update the annual health records for all employees",
        "The annual health check program starts next Monday",
        "Flu shots are free for office staff this season",
        "After surgery he is recovering well and may leave the hospital this week",
        // ---- Legal / administrative (10) ----
        "The business registration procedure is completed within three business days",
        "The enterprise registration certificate was issued with a new tax code",
        "The charter capital was increased to ten billion dong",
        "Official letter twenty five requires the results report by the twentieth",
        "Every document for signature must be reviewed by the legal team first",
        "The resignation letter must be submitted thirty days in advance",
        "The business certificate needs reissuing with the new office address",
        "The legal representative of the company is the director per the charter",
        "The project legal file includes the construction permit and land papers",
        "The appointment decision takes effect on the signing date",
        // ---- Emotions / relationships (8) ----
        "Thank you for always standing by and supporting me",
        "When we all try together everything will go smoothly",
        "I really appreciate the care and sharing from the whole team",
        "The team spirit this week is much better than before",
        "Please feel free to leave the task to me",
        "Stay calm and take it step by step, things will be fine",
        "I believe the team's effort will be well recognized",
        "Take care of yourself and keep the positive spirit",
        // ---- Forms / internal governance (10) ----
        "The employee evaluation form adds a development goal section this cycle",
        "The summary table matches the accounting department report",
        "Please update the new staff list in the system before Friday",
        "The team task list is updated every day",
        "The handover file includes acceptance minutes and as built drawings",
        "The work journal helps the team track progress more clearly",
        "The quarterly target is broken into monthly goals",
        "Each department's spending limit is approved by the annual budget",
        "The weekly report is due before five pm every Friday",
        "All files must be archived within the required retention period",
    ];

    for s in sentences {
        add_splits_from(1, s, &mut add);
    }

    // ------------------------------------------------------------------
    // 2. ENGLISH IDIOMS (chỉ mục thật) — chia tại mọi điểm.
    // ------------------------------------------------------------------
    let idioms: &[&str] = &[
        "A piece of cake",
        "Break a leg",
        "Hit the ground running",
        "Beat around the bush",
        "Bite the bullet",
        "Call it a day",
        "Cut to the chase",
        "Get the ball rolling",
        "Get out of hand",
        "Go the extra mile",
        "Back to the drawing board",
        "On the same page",
        "In the long run",
        "At the eleventh hour",
        "Once in a blue moon",
        "Under the weather",
        "The ball is in your court",
        "A blessing in disguise",
        "Actions speak louder than words",
        "Add insult to injury",
        "Against the clock",
        "Barking up the wrong tree",
        "Burn the midnight oil",
        "By the book",
        "Cost an arm and a leg",
        "Cry over spilt milk",
        "Every cloud has a silver lining",
        "Fortune favors the brave",
        "Give someone the benefit of the doubt",
        "Hit the nail on the head",
        "In the heat of the moment",
        "Jump on the bandwagon",
        "Keep something at bay",
        "Kill two birds with one stone",
        "Leave no stone unturned",
        "Let sleeping dogs lie",
        "Look before you leap",
        "Make a long story short",
        "Miss the boat",
        "No pain no gain",
        "Not a bed of roses",
        "On thin ice",
        "Out of the frying pan into the fire",
        "Practice makes perfect",
        "Pull yourself together",
        "Put all your eggs in one basket",
        "Rain cats and dogs",
        "Rome was not built in a day",
        "See eye to eye",
        "Speak of the devil",
        "Spill the beans",
        "Take it with a grain of salt",
        "The best of both worlds",
        "The early bird catches the worm",
        "The last straw",
        "The tip of the iceberg",
        "Through thick and thin",
        "Time is money",
        "To be on cloud nine",
        "To make matters worse",
        "When it rains it pours",
        "Wrap your head around something",
        "Your guess is as good as mine",
        "A dime a dozen",
        "Better late than never",
        "Curiosity killed the cat",
        "Go back to square one",
        "Haste makes waste",
        "It takes two to tango",
        "Learn the ropes",
        "On the ball",
        "Pull the plug",
        "Raise the bar",
        "Strike while the iron is hot",
        "Take the bull by the horns",
        "Think outside the box",
        "Up in the air",
        "When pigs fly",
        "You can't judge a book by its cover",
        // Vòng 53 (chỉ mục thật)
        "Birds of a feather flock together",
        "Don't count your chickens before they hatch",
        "Every dog has its day",
        "A friend in need is a friend indeed",
        "Two heads are better than one",
        "The devil is in the details",
        "Slow and steady wins the race",
        "Better safe than sorry",
        "Easy come easy go",
        "Practice what you preach",
        "The squeaky wheel gets the grease",
        "You reap what you sow",
        "A picture is worth a thousand words",
        "Barking dogs seldom bite",
        "Honesty is the best policy",
    ];

    for s in idioms {
        // Vòng 52: thành ngữ neo khoá ≥ 2 từ — khoá 1 từ generic vô nghĩa.
        add_splits_from(2, s, &mut add);
    }

    // ------------------------------------------------------------------
    // 3. NHÓM CURATED DOMAIN (bảng cũ regression-tested — giữ nguyên):
    //    office / tech-git / tech_verbs / business / OS APIs / Gen Z.
    // ------------------------------------------------------------------
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

    // ------------------------------------------------------------------
    // 4. CẶP CURATED + MA TRẬN NGỮ NGHĨA-CONTROLLED.
    // ------------------------------------------------------------------
    // 4a. Cặp curated (mở rộng).
    for &(ctx, pred, weight) in &[
        ("thank you", "very much", 100),
        ("thank you", "for your email", 96),
        ("thank you", "for your support", 94),
        ("thank you", "for your time", 92),
        ("thank you", "in advance", 92),
        ("thank you again", "for your help", 90),
        ("thanks", "a lot", 96),
        ("thanks", "in advance", 92),
        ("thanks for", "your quick response", 90),
        ("thanks for", "letting me know", 88),
        ("looking forward to", "hearing from you", 100),
        ("looking forward to", "working with you", 94),
        ("looking forward to", "your feedback", 92),
        ("looking forward", "to hearing from you soon", 94),
        ("please let me know", "if you have any questions", 94),
        ("please let me know", "your availability", 92),
        ("please let me know", "if the schedule works", 90),
        ("please find", "attached the report", 94),
        ("please find", "the updated document", 90),
        ("please review", "the attached document", 92),
        ("please review", "and share your feedback", 88),
        ("could you please", "send me the final version", 90),
        ("could you please", "confirm your attendance", 88),
        ("could you please", "let me know", 88),
        ("i am writing to", "follow up on", 92),
        ("i am writing to", "confirm", 88),
        ("i am writing to", "inform you that", 88),
        ("i would like to", "request your approval", 90),
        ("i would like to", "schedule a meeting", 88),
        ("i would like to", "follow up on", 86),
        ("we would like to", "invite you to", 88),
        ("we would like to", "inform you that", 86),
        ("we apologize for", "the inconvenience", 94),
        ("we apologize", "for the delay", 92),
        ("i apologize for", "the late reply", 88),
        ("in order to", "ensure", 92),
        ("in order to", "avoid any confusion", 86),
        ("in order to", "meet the deadline", 86),
        ("as soon as", "possible", 94),
        ("as soon as", "I have an update", 88),
        ("as discussed", "I have attached", 88),
        ("as per", "your request", 88),
        ("as per", "our agreement", 86),
        ("kindly note", "that the deadline", 86),
        ("kindly confirm", "your attendance", 88),
        ("kindly review", "the attached document", 86),
        ("just a quick", "reminder that", 88),
        ("just checking in", "to see if", 86),
        ("no cap", "that was the cleanest code refactor ever", 96),
        ("sounds good", "to me", 92),
        ("looks good", "to me, approved and ready to merge", 94),
        ("sounds like", "a plan", 88),
        ("on my way", "to the office", 86),
        ("out of office", "until next Monday", 86),
        ("eod", "today please", 84),
        ("asap", "please", 88),
        ("fyi", "the meeting is moved", 84),
        ("imo", "we should prioritize", 82),
        ("btw", "the client confirmed", 82),
        ("talk soon", "and have a great day", 84),
        ("best regards", "and have a great week", 86),
        ("kind regards", "and thank you again", 84),
        ("get well soon", "and take your time", 84),
        ("happy to help", "any time", 86),
        ("happy birthday", "and have a fantastic day", 86),
        ("congratulations on", "the promotion", 88),
        ("congratulations", "on the well deserved success", 84),
        ("the meeting is", "scheduled for", 90),
        ("the meeting has been", "rescheduled to", 88),
        ("the deadline is", "this Friday", 90),
        ("the deadline for", "the final submission", 86),
        ("the report is", "attached for your review", 86),
        ("the invoice has been", "processed", 86),
        ("the payment has been", "transferred", 88),
        ("the issue has been", "resolved", 90),
        ("the issue has been", "escalated to our team", 84),
        ("your request has been", "received and processed", 86),
        ("your order has been", "shipped", 88),
        ("we are currently", "looking into this issue", 88),
        ("we are pleased to", "offer you", 88),
        ("we look forward to", "a long term partnership", 86),
        ("we value", "your feedback", 88),
        ("do not hesitate", "to contact me", 90),
        ("feel free to", "reach out", 88),
        ("let me know", "if you need anything", 90),
        ("let me know", "when you are available", 86),
        ("keep me", "posted on the progress", 86),
        ("i will keep", "you posted", 88),
        ("i will get back", "to you shortly", 88),
        ("i have attached", "the document", 90),
        ("i have forwarded", "your request", 86),
        ("per our", "last conversation", 84),
        ("at your", "earliest convenience", 86),
        ("by end of", "day today", 88),
        ("by the end of", "this week", 88),
        ("on track", "and within budget", 86),
        ("behind schedule", "due to", 84),
        ("ahead of", "schedule", 84),
        ("stay tuned", "for updates", 82),
        ("long story", "short", 86),
        ("works for", "me", 88),
        ("dear", "sir or madam", 92),
        ("dear", "mr", 88),
        ("dear", "ms", 86),
        ("dear team", "and colleagues", 84),
        ("dear all", "please be informed", 84),
        ("does that", "work for you", 86),
        ("see you", "at the meeting", 84),
        ("see you", "next week", 84),
        ("have a great", "weekend", 88),
        ("have a good", "one", 84),
    ] {
        add(ctx, pred, weight);
    }

    // Vòng 53: curated mở rộng — banking/insurance/logistics/real estate
    for &(ctx, pred, weight) in &[
        ("the lease", "for another year", 86),
        ("the rental agreement", "before signing", 82),
        ("the monthly fee", "covers utilities", 82),
        ("the ownership transfer", "will be notarized", 76),
        ("the property value", "in this area", 78),
        ("please pay the deposit", "to hold the apartment", 76),
        ("a savings account", "with a six month term", 78),
        ("the interest rate", "for this term", 82),
        ("my statement", "needs investigation", 76),
        ("please confirm the", "one time password", 80),
        ("my credit card", "was locked", 78),
        ("the home loan application", "needs", 78),
        ("the bank will", "disburse the loan", 76),
        ("the annual card fee", "is waived", 74),
        ("my life insurance", "premium is due", 76),
        ("the claims process", "usually takes", 78),
        ("please submit the claim", "with the receipts", 76),
        ("the payout amount", "is calculated", 74),
        ("the customs declaration", "was cleared", 80),
        ("the shipment is expected", "to arrive", 84),
        ("the import duty", "is calculated", 76),
        ("the goods will be moved", "to the warehouse", 76),
        ("the end of month inventory", "is slightly higher", 74),
        ("please recount the", "quantity of goods", 74),
        ("the shipping fee", "includes", 78),
        ("the carrier commits", "to delivering", 76),
        ("the damaged package", "needs an inspection report", 72),
        ("the delivery schedule", "for the whole month", 78),
        ("the freight cost", "increased", 76),
        ("the bill of lading", "and commercial invoice", 74),
        ("the production line", "will stop for maintenance", 78),
        ("the factory output", "this quarter", 78),
        ("the replacement parts", "will arrive", 76),
        ("safety equipment", "is mandatory", 78),
        ("the technical issue", "was fixed", 78),
        ("the defect rate", "of this batch", 76),
        ("the design capacity", "of the plant", 74),
        ("the quality control process", "consists of", 76),
        ("the bid documents", "must be submitted", 78),
        ("this tender", "requires", 76),
        ("the selection result", "will be announced", 76),
        ("the payment terms", "include", 80),
        ("the memorandum", "signed yesterday", 76),
        ("the warranty clause", "before signing", 74),
        ("the contract value", "is adjusted", 76),
        ("the contract liquidation", "will be done", 72),
        ("the buyer requests", "a one week extension", 72),
        ("any contract change", "must be recorded", 74),
        ("the press release", "will go out", 76),
        ("the detailed script", "for the video", 74),
        ("the communication plan", "for next quarter", 78),
        ("the campaign reached", "ten thousand interactions", 74),
        ("please review the content", "before posting", 76),
        ("the brand identity", "has been updated", 74),
        ("our research project", "has been approved", 76),
        ("the survey shows", "seventy percent", 78),
        ("the research method", "must clearly describe", 72),
        ("my thesis defense", "is scheduled for", 74),
        ("the scholarship", "is awarded based on", 74),
        ("the project finished", "the foundation", 72),
        ("construction progress", "is at seventy percent", 74),
        ("site safety", "is the number one priority", 76),
        ("the contractor commits", "to handover", 72),
        ("i would like to book", "a double room", 78),
        ("my flight arrives", "at the airport", 82),
        ("the round trip ticket", "has been issued", 76),
        ("check in starts", "at two pm", 78),
        ("the detailed itinerary", "will be sent", 74),
        ("we cancelled the booking", "due to a schedule conflict", 72),
        ("the lab results", "will be ready", 78),
        ("the doctor scheduled", "a follow up visit", 76),
        ("this prescription", "needs to be filled", 72),
        ("health insurance covers", "seventy percent", 74),
        ("the annual health check", "starts next Monday", 74),
        ("the business registration", "procedure is completed", 74),
        ("the enterprise registration", "certificate was issued", 72),
        ("the charter capital", "was increased", 74),
        ("every document for signature", "must be reviewed", 72),
        ("the legal representative", "of the company", 76),
        ("the appointment decision", "takes effect", 74),
        ("the employee evaluation form", "adds a", 72),
        ("the summary table", "matches", 76),
        ("please update the staff list", "in the system", 74),
        ("the quarterly target", "is broken into", 72),
        ("the weekly report", "is due before", 76),
        ("all files must be", "archived within", 74),
    ] {
        add(ctx, pred, weight);
    }

    // 4b. Ma trận subject × predicate (mọi tổ hợp tự nhiên).
    for subject in [
        "I",
        "We",
        "The team",
        "Our team",
        "He",
        "She",
        "They",
        "The client",
    ] {
        for predicate in [
            "will get back to you shortly",
            "will keep you posted on the progress",
            "has reviewed the attached document",
            "has approved the final version",
            "is currently working on the fix",
            "is out of office until Monday",
            "will join the call at three o'clock",
            "has been notified about the change",
            "will handle the follow up email",
            "is responsible for the next milestone",
            "will prepare the agenda for tomorrow",
            "has shared the updated timeline",
            "has circulated the meeting minutes",
            "is waiting for the client confirmation",
            "has double checked the numbers",
            "will rearrange the schedule next week",
            "has agreed to support this task",
            "will attend the training session tomorrow",
            "has escalated the issue to management",
        ] {
            add(subject, predicate, 88);
        }
    }

    // 4c. Ma trận verb × object (business collocations).
    let verb_objects: &[(&str, &[&str])] = &[
        (
            "send",
            &[
                "the report by Friday",
                "me the updated file",
                "the invoice",
                "a calendar invite",
                "the final draft",
                "your feedback",
                "the signed contract",
                "the payment confirmation",
            ],
        ),
        (
            "please send",
            &[
                "the report by Friday",
                "the updated file",
                "the invoice",
                "a calendar invite",
                "the final draft",
                "your feedback",
            ],
        ),
        (
            "review",
            &[
                "the attached document",
                "the latest draft",
                "the proposal",
                "the terms and conditions",
                "the meeting notes",
                "the quarterly report",
            ],
        ),
        (
            "please review",
            &[
                "the attached document",
                "the latest draft",
                "the proposal",
                "the meeting notes",
                "the quarterly report",
            ],
        ),
        (
            "confirm",
            &[
                "your attendance",
                "the delivery date",
                "the meeting time",
                "receipt of this email",
                "the payment details",
                "the final numbers",
            ],
        ),
        (
            "please confirm",
            &[
                "your attendance",
                "the delivery date",
                "the meeting time",
                "receipt of this email",
                "the payment details",
            ],
        ),
        (
            "schedule",
            &[
                "a quick call",
                "a follow up meeting",
                "a product demo",
                "the next review",
                "an interview",
                "a training session",
            ],
        ),
        (
            "follow up",
            &[
                "on the open items",
                "with the client",
                "on the pending invoice",
                "on our last conversation",
                "with the vendor",
                "on the support ticket",
            ],
        ),
        (
            "update",
            &[
                "the project status",
                "the contact list",
                "the timeline",
                "the budget forecast",
                "the order details",
                "the team on progress",
            ],
        ),
        (
            "prepare",
            &[
                "the monthly report",
                "the meeting agenda",
                "a cost estimate",
                "the presentation slides",
                "the project plan",
                "the minutes of the meeting",
            ],
        ),
        (
            "share",
            &[
                "the updated document",
                "your availability",
                "the meeting notes",
                "the feedback",
                "the contact details",
                "the results",
            ],
        ),
        (
            "complete",
            &[
                "the pending tasks",
                "the onboarding steps",
                "the annual review",
                "the form and return it",
                "the payment process",
            ],
        ),
        (
            "arrange",
            &[
                "a handover session",
                "the delivery schedule",
                "a training for the new staff",
                "the transfer of documents",
                "an introduction call",
            ],
        ),
        (
            "reconcile",
            &[
                "the outstanding balance",
                "the invoices for this quarter",
                "the accounts before month end",
                "the payment records",
            ],
        ),
        (
            "escalate",
            &[
                "the issue to management",
                "the delay to the vendor",
                "the request to the support team",
            ],
        ),
        (
            "hand over",
            &[
                "the duties to the successor",
                "the documents to the receiving team",
                "the project to the new lead",
            ],
        ),
    ];
    for (verb, objects) in verb_objects {
        for obj in *objects {
            add(verb, obj, 88);
            add(&format!("kindly {verb}"), obj, 78);
            add(&format!("could you {verb}"), obj, 78);
        }
    }

    // 4d. Deadline frames: action × deadline.
    for action in [
        "send the report",
        "submit the form",
        "confirm your attendance",
        "review the document",
        "complete the payment",
        "sign the contract",
        "send your feedback",
        "submit the expenses",
        "book the meeting room",
        "finalize the proposal",
        "update the status",
        "share the slides",
    ] {
        for deadline in [
            "by end of day",
            "by Friday",
            "by next Monday",
            "by noon today",
            "before the meeting",
            "by the end of the week",
            "within 24 hours",
            "by the deadline",
            "as soon as possible",
            "before month end",
        ] {
            add(action, deadline, 84);
        }
    }

    // 4e. Greetings × wishes.
    for greeting in [
        "Happy birthday",
        "Happy new year",
        "Have a great",
        "Have a wonderful",
        "Wishing you",
    ] {
        for wish in [
            "day ahead",
            "weekend",
            "a wonderful day",
            "a relaxing weekend",
            "all the best",
            "a speedy recovery",
            "a happy holiday season",
            "success in your new role",
            "a safe trip",
            "great success",
        ] {
            add(greeting, wish, 82);
        }
    }

    // 4f. Appointment frames: purpose × time.
    for purpose in [
        "the kickoff meeting",
        "the quarterly review",
        "a quick sync",
        "the performance review",
        "the product demo",
        "the client meeting",
        "the team standup",
        "the annual audit",
        "the training session",
        "the onboarding call",
    ] {
        for time in [
            "at ten o'clock",
            "tomorrow morning",
            "next Monday",
            "at two this afternoon",
            "on Thursday",
            "at the end of the week",
            "on the fifteenth",
            "after lunch",
            "this Friday",
            "next week",
        ] {
            add(purpose, time, 82);
        }
    }

    // 4g. Cross-product đã LỌC: chỉ prefix infinitive-frame (mọi tổ hợp với
    //     verb-phrase đều tự nhiên — bỏ prefix ngữ-cảnh xã giao gây câu rác).
    let infinitive_prefixes = [
        "i would like to",
        "could you please",
        "please make sure to",
        "we need to ensure",
        "it is important to",
        "don't forget to",
        "i am happy to",
        "we are pleased to",
        "can you please check",
        "we have successfully",
        "please remember that",
        "i will follow up on",
        "we recommend that you",
        "we are currently working on",
        "to resolve this issue",
        "it is essential that",
        "the team agreed to",
        "please confirm that",
        "make sure that the",
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

    for &p in &infinitive_prefixes {
        for &s in &subjects_pred {
            add(p, s, 84);
        }
    }

    println!("Tổng số mẫu câu / cụm từ tiếng Anh đã sinh: {}", set.len());

    let mut out_str = String::with_capacity(set.len() * 60);
    out_str.push_str("# Ngữ cảnh tiếng Anh (Next-Word / Phrase Prediction - P6)\n");
    out_str
        .push_str("# Vòng 52: câu thật chia mọi ranh giới từ + idioms + cặp curated + ma trận\n");
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

/// Thêm mọi cặp (tiền tố → phần còn lại) của câu `s`:
/// `w1 → w2…wn`, `w1 w2 → w3…wn`, … Khoá ≥ 1 từ, dự đoán ≥ 1 từ.
/// Weight theo số từ khoá: 1 từ = 70, 2 từ = 82, ≥3 từ = 92.
fn add_splits_from(min_split: usize, s: &str, add: &mut impl FnMut(&str, &str, u32)) {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() < min_split + 1 {
        return;
    }
    for split in min_split..words.len() {
        let key = words[..split].join(" ");
        let rest = words[split..].join(" ");
        let weight = match split {
            1 => 70,
            2 => 82,
            _ => 92,
        };
        add(&key, &rest, weight);
    }
}
