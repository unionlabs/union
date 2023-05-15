# `gnark` Security Policy

## Overview

This document explains the gnark team's process for handling issues reported and what to expect in return.

## Reporting a Security Bug

All security bugs in gnark distribution should be reported by email to gnark@consensys.net.

Your email will be acknowledged within 7 days, and you'll be kept up to date with the progress until resolution. Your issue will be fixed or made public within 90 days.

If you have not received a reply to your email within 7 days, please follow up with the gnark team again at gnark@consensys.net. 

Note that we do not currently run any bug bounty program.

## Tracks

Depending on the nature of your issue, it will be categorized as an issue in the **PUBLIC**, **PRIVATE**, or **URGENT** track.

### PUBLIC

Issues in the **PUBLIC** track affect niche configurations, have very limited impact, or are already widely known.

**PUBLIC** track issues are fixed on the develop branch, and get backported to the next scheduled minor releases. The release announcement includes details of these issues, but there is no pre-announcement.

### PRIVATE

Issues in the **PRIVATE** track are violations of committed security properties.

**PRIVATE** track issues are fixed in the next scheduled minor releases , and are kept private until then.

Three to seven days before the release, a pre-announcement is sent to [`gnark-announce`] and [@gnark_team], announcing the presence of a security fix in the upcoming releases, and which component in gnark is affected; compiler, constraint system or proof system (but not disclosing any more details).

### URGENT

**URGENT** track issues are a threat to the gnark ecosystem's integrity, or are being actively exploited in the wild leading to severe damage.

**URGENT** track issues are fixed in private, and trigger an immediate dedicated security release, possibly with no pre-announcement.

## Flagging Existing Issues as Security-related

If you believe that an existing issue is security-related, we ask that you send an email to gnark@consensys.net. The email should include the issue ID and a short description of why it should be handled according to this security policy.

## Disclosure Process

The gnark project uses the following disclosure process:

* Once the security report is received it is assigned a primary handler. This person coordinates the fix and release process.
* The issue is confirmed and a list of affected components is determined.
* Code is audited to find any potential similar problems.
* Fixes are prepared for the two most recent major releases and the head/master revision. Fixes are prepared for the two most recent major releases and merged to head/master.
* On the date that the fixes are applied, announcements are sent to [`gnark-announce`] and [@gnark_team].

This process can take some time, especially when coordination is required with maintainers of other projects. Every effort will be made to handle the bug in as timely a manner as possible, however it's important that we follow the process described above to ensure that disclosures are handled consistently.

## Receiving Security Updates

The best way to receive security announcements is to subscribe to the [`gnark-announce`] mailing list. Any messages pertaining to a security issue will be prefixed with \[security\].

[`gnark-announce`]: https://groups.google.com/g/gnark-announce
[@gnark_team]: https://twitter.com/gnark_team