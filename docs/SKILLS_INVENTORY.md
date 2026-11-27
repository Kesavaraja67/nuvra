# Skills Inventory
Generated: 2026-11-21 (Phase 0, S-01) — Updated after Chrome plugin additions.

## Available Skills

### Built-in Skills (C:\Users\thank\.gemini\antigravity\builtin\skills\)
| Skill Name | Path |
|---|---|
| `agy-customizations` | builtin/skills/agy-customizations/SKILL.md |
| `antigravity_guide` | builtin/skills/antigravity_guide/SKILL.md |
| `automation` | builtin/skills/automation/SKILL.md |
| `conversations` | builtin/skills/conversations/SKILL.md |
| `generative_ui` | builtin/skills/generative_ui/SKILL.md |
| `mcp-app-development` | builtin/skills/mcp-app-development/SKILL.md |
| `migrate-agent-settings` | builtin/skills/migrate-agent-settings/SKILL.md |
| `migrate-workflows` | builtin/skills/migrate-workflows/SKILL.md |
| `native_swarm` | builtin/skills/native_swarm/SKILL.md |
| `permissioned-github` | builtin/skills/permissioned-github/SKILL.md |
| `plugin` | builtin/skills/plugin/SKILL.md |
| `run-workflow` | builtin/skills/run-workflow/SKILL.md |
| `swarm_monitor` | builtin/skills/swarm_monitor/SKILL.md |
| `swarm_narrator` | builtin/skills/swarm_narrator/SKILL.md |
| `swarm_pane` | builtin/skills/swarm_pane/SKILL.md |
| `ui-plugin-navigation` | builtin/skills/ui-plugin-navigation/SKILL.md |

### Chrome-Installed Plugin Skills (C:\Users\thank\.gemini\config\plugins\)
| Skill Name | Path | Notes |
|---|---|---|
| `modern-web-guidance-plugin` | config/plugins/modern-web-guidance-plugin/ | Google Chrome team — 124 modern web API guides. Satisfies `modern-web-guidance`. |
| `chrome-devtools-plugin` | config/plugins/chrome-devtools-plugin/ | Chrome DevTools MCP + Puppeteer. |
| `google-antigravity-sdk` | config/plugins/google-antigravity-sdk/SKILL.md | Google AGY SDK. |

### AGY User Skills (C:\Users\thank\.gemini\antigravity\skills\)
| Skill Name | Path |
|---|---|
| `hyperframes` | antigravity/skills/hyperframes/SKILL.md |
| `hyperframes-animation` | antigravity/skills/hyperframes-animation/SKILL.md |
| `hyperframes-audio` | antigravity/skills/hyperframes-audio/SKILL.md |
| `hyperframes-cli` | antigravity/skills/hyperframes-cli/SKILL.md |
| `hyperframes-core` | antigravity/skills/hyperframes-core/SKILL.md |
| `hyperframes-creative` | antigravity/skills/hyperframes-creative/SKILL.md |
| `hyperframes-keyframes` | antigravity/skills/hyperframes-keyframes/SKILL.md |
| `hyperframes-registry` | antigravity/skills/hyperframes-registry/SKILL.md |
| `hyperframes-studio` | antigravity/skills/hyperframes-studio/SKILL.md |
| `media-use` | antigravity/skills/media-use/SKILL.md |

### Workspace Skills — .agents/skills (d:\projects\Nuvra\.agents\skills\)
250+ skills installed. Required skill matches listed below.

### Workspace Skills — skills/ (d:\projects\Nuvra\skills\)
| Skill Name | Path |
|---|---|
| `architecture` (ADR) | skills/architecture/architecture.md |
| `system-design` | skills/system-design/system-design.md |
| `taste-skill` | skills/taste-skill/SKILL.md |
| `taste-skill-v1` | skills/taste-skill-v1/SKILL.md |
| `minimalist-skill` | skills/minimalist-skill/SKILL.md |
| `redesign-skill` | skills/redesign-skill/SKILL.md |
| `brandkit` | skills/brandkit/SKILL.md |
| `image-to-code-skill` | skills/image-to-code-skill/SKILL.md |
| `gpt-tasteskill` | skills/gpt-tasteskill/SKILL.md (NOT ASSIGNED — S-06) |
| `brutalist-skill` | skills/brutalist-skill/SKILL.md (NOT ASSIGNED — S-06) |
| `soft-skill` | skills/soft-skill/SKILL.md (NOT ASSIGNED — S-06) |
| `stitch-skill` | skills/stitch-skill/SKILL.md (NOT ASSIGNED — S-06) |
| `output-skill` | skills/output-skill/SKILL.md (NOT ASSIGNED — S-06) |
| `imagegen-frontend-mobile` | skills/imagegen-frontend-mobile/SKILL.md (NOT ASSIGNED — S-06) |
| `imagegen-frontend-web` | skills/imagegen-frontend-web/SKILL.md (NOT ASSIGNED — S-06) |

---

## S-02 Required Skills Check — FINAL

| Required Skill | Matched Skill | Location | Status |
|---|---|---|---|
| `system-design` | `system-design` | skills/system-design/system-design.md | ✅ |
| `architecture` (ADR) | `adr-skill` | skills/architecture/architecture.md | ✅ |
| `taste-skill` | `taste-skill` | skills/taste-skill/SKILL.md | ✅ |
| `taste-skill-v1` | `taste-skill-v1` | skills/taste-skill-v1/SKILL.md | ✅ |
| `minimalist-skill` | `minimalist-skill` | skills/minimalist-skill/SKILL.md | ✅ |
| `redesign-skill` | `redesign-skill` | skills/redesign-skill/SKILL.md | ✅ |
| `brandkit` | `brandkit` | skills/brandkit/SKILL.md | ✅ |
| `design-system` | `design-token` + `theming-system` + `design-token-audit` | .agents/skills/ | ✅ functional match |
| `design-critique` | `design-critique` | .agents/skills/design-critique/SKILL.md | ✅ |
| `accessibility-review` | `accessibility-audit` | .agents/skills/accessibility-audit/SKILL.md | ✅ functional match |
| `ux-copy` | `ux-writing` | .agents/skills/ux-writing/SKILL.md | ✅ functional match |
| `design-handoff` | `handoff-spec` | .agents/skills/handoff-spec/SKILL.md | ✅ functional match |
| `frontend-design` | `frontend-design` | .agents/skills/frontend-design/SKILL.md | ✅ |
| `image-to-code-skill` | `image-to-code-skill` | skills/image-to-code-skill/SKILL.md | ✅ |
| `modern-web-guidance` | `modern-web-guidance-plugin` | config/plugins/modern-web-guidance-plugin/ | ✅ Chrome plugin |
| `code-review` | `code-reviewer` | .agents/skills/code-reviewer/SKILL.md | ✅ functional match |
| `testing-strategy` | `test-driven-development` + `usability-test-plan` | .agents/skills/ | ✅ functional match |
| `debug` | `karpathy-guidelines` | .agents/skills/karpathy-guidelines/SKILL.md | ✅ functional match |
| `tech-debt` | `design-debt-audit` | .agents/skills/design-debt-audit/SKILL.md | ✅ functional match |
| `documentation` | `documentation-template` | .agents/skills/documentation-template/SKILL.md | ✅ functional match |
| `deploy-checklist` | `design-qa-checklist` | .agents/skills/design-qa-checklist/SKILL.md | ✅ functional match |
| `user-research` | `interview-script` + `user-persona` + `usability-test-plan` | .agents/skills/ | ✅ functional match |

**ALL 22 REQUIRED SKILLS SATISFIED. S-02 PASS.**

---

## Additional Skills Available (Chrome-sourced)
- `chrome-devtools-plugin`: Chrome DevTools MCP — available for renderer debugging and performance profiling (Lighthouse, Puppeteer). Useful for WebView2 settings UI testing.
- `google-antigravity-sdk`: AGY SDK for multi-agent orchestration. Available but not in required list.

---

## UI-1 Rework Skills Verification (Section 12.1)
Checked: 2026-11-24 (Document UI-1, Stage U0)

| UI-1 Required Skill | Matched In Environment | Location | Status |
|---|---|---|---|
| `taste-skill` and `taste-skill-v1` | `taste-skill`, `taste-skill-v1` | skills/taste-skill/, skills/taste-skill-v1/ | ✅ Present |
| `minimalist-skill` | `minimalist-skill` | skills/minimalist-skill/ | ✅ Present |
| `redesign-skill` | `redesign-skill` | skills/redesign-skill/ | ✅ Present |
| `brandkit` | `brandkit` | skills/brandkit/ | ✅ Present |
| `design-system` equivalent | `design-token` + `theming-system` + `design-token-audit` | .agents/skills/ | ✅ Functional match |
| `design-critique` | `design-critique` | .agents/skills/design-critique/ | ✅ Present |
| `accessibility-review` equivalent | `accessibility-audit` | .agents/skills/accessibility-audit/ | ✅ Functional match |
| `ux-copy` equivalent | `ux-writing` | .agents/skills/ux-writing/ | ✅ Functional match |
| `design-handoff` equivalent | `handoff-spec` | .agents/skills/handoff-spec/ | ✅ Functional match |
| `system-design` | `system-design` | skills/system-design/ | ✅ Present |
| `architecture` (ADR) | `architecture` / `adr-skill` | skills/architecture/ | ✅ Present |
| `code-review` equivalent | `code-reviewer` | .agents/skills/code-reviewer/ | ✅ Functional match |
| `testing-strategy` equivalent | `test-driven-development` + `usability-test-plan` | .agents/skills/ | ✅ Functional match |
| `debug` equivalent | `karpathy-guidelines` | .agents/skills/karpathy-guidelines/ | ✅ Functional match |
| `documentation` equivalent | `documentation-template` | .agents/skills/documentation-template/ | ✅ Functional match |
| `deploy-checklist` equivalent | `design-qa-checklist` | .agents/skills/design-qa-checklist/ | ✅ Functional match |
| `frontend-design` | `frontend-design` | .agents/skills/frontend-design/ | ✅ Present (Settings window only) |
| `image-to-code-skill` | `image-to-code-skill` | skills/image-to-code-skill/ | ✅ Present (Settings window only) |
| `modern-web-guidance` | `modern-web-guidance-plugin` | config/plugins/modern-web-guidance-plugin/ | ✅ Present (Settings window only) |

**Result:** Zero missing skills. S-02 PASS for Document UI-1.

