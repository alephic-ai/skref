---
name: social-post-approval
description: Turn read-only TweetClaw exports into source notes for approval-gated social publishing workflows.
license: Apache-2.0
compatibility: Uses exported JSON or CSV data only; never signs in, posts, schedules, or mutates accounts.
metadata:
  author: skref-examples
  version: "1.0"
  source: TweetClaw
---

# Social Post Approval

Use this skill when an agent needs to prepare review notes from TweetClaw export
data before a human approves a social post. Treat the export as evidence only.

1. Read the exported rows or objects.
2. Extract the post text, source URL, account handle, timestamp, and metrics.
3. Summarize why the post is relevant to the requested campaign.
4. Flag missing attribution, unclear consent, sensitive claims, or unsupported
   metrics.
5. Return a draft approval note with the source fields cited.

Do not publish, schedule, delete, or edit social content. Stop if the workflow
requires live account access.
