# HOSxp Dash

```
██╗  ██╗ ██████╗  ██████╗██╗  ██╗██████╗ ██████╗  █████╗  ██████╗██╗  ██╗
██║  ██║██╔═══██╗██╔════╝╚██╗██╔╝██╔══██╗██╔══██╗██╔══██╗██╔════╝██║  ██║
███████║██║   ██║███████╗ ╚███╔╝ ██████╔╝██║  ██║███████║███████╗███████║
██║  ██║██║   ██║╚════██║ ███╔╝  ██╔═══╝ ██║  ██║██╔══██║╚════██║██║  ██║
██║  ██║╚██████╔╝██████╔╝██╔██╗  ██║     ██████╔╝██║  ██║██████╔╝██║  ██║
╚═╝  ╚═╝ ╚═════╝ ╚═════╝╚═╝ ╚═╝╚═╝╚═════╝ ╚═╝  ╚═╝╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝
```

---

## ◆ PULSE

The numbers live in HOSxP; the question lives in the meeting room.
HOSxp Dash connects directly to the hospital's MySQL database, read-only,
and turns `opitemrece` into the picture a pharmacy meeting needs: the
year's drug usage, the top products, and one drug's monthly trend with
its peak month called out. No export, no spreadsheet archaeology - the
Rust backend asks the database, and ECharts draws the answer.

| Trend ▣ | Top drugs ▣ | Year view ▣ | Direct read ▣ |
|---|---|---|---|

*The dashboard - connect, search, trend, rank - is sealed.*

> Built with Tauri 2 + Vue 3 + Pinia, drawn by Apache ECharts, read from
> HOSxP MySQL by a Rust `sqlx` backend - nothing written back.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One clone, one install, one command.

```
⟫ git clone https://github.com/suradet-ps/hosxp-dash.git
⟫ cd hosxp-dash
⟫ bun install
⟫ bun run tauri dev
```

The release artifact: `⟫ bun run tauri build` - output lands in
`src-tauri/target/release/bundle`.

<details>
<summary>Prerequisites</summary>

- [Bun](https://bun.sh/) (or Node.js 18+)
- [Rust](https://www.rust-lang.org/) with Cargo
- Tauri system dependencies for your OS

</details>

---

## ◆ ANATOMY

One connection, one table, several honest charts.

- **Connects** - the settings dialog takes host, port, database, and
  credentials; the Rust backend holds the pool and asks the questions
  directly - no intermediate service, no export pipeline.
- **Asks** - the queries are explicit: the available years from
  `opitemrece`, the dashboard bundle for a year, top drugs by quantity,
  and one drug's monthly quantities with the peak month computed in the
  query itself.
- **Searches** - the drug search bar finds items by name or code and
  hands the selection to the trend chart.
- **Draws** - ECharts renders the month-by-month curve and the top-drugs
  ranking; the KPI bar summarizes the year in one glance.
- **Stays** - the dashboard is desktop-native through Tauri: local,
  fast, and presentable on a hospital machine that never sees the
  public internet.

---

## ◆ RITUALS

**The core ceremony** - the pharmacy meeting prep:

1. Open the app, connect to HOSxP. One configuration, remembered.
2. Pick the year. The dashboard answers: KPIs, top drugs, and the
   usage shape of the year.
3. Search a drug of interest; its monthly trend renders with the peak
   month marked.
4. Close the app. Nothing was written, nothing was exported, nothing
   left the machine.

**The ceremony of the direct read** - the chart is only as good as the
query, and the query is visible in the backend: year, quantity, peak
month - computed in SQL, drawn in ECharts, no middleman to corrupt the
number.

**The ceremony of restraint** - read-only by construction: the backend
holds a connection for questions, not for writes. The dashboard
presents the record; it never edits it.

---

## ◆ ECHOES

**Where this artifact is heading**

```
connect ▸ HOSxP MySQL connection settings ─────────────────────────── ▸ sealed
ask      ▸ years, dashboard bundle, top drugs, drug trend ──────────── ▸ sealed
draw     ▸ ECharts trend + ranking, KPI bar ────────────────────────── ▸ sealed
search   ▸ drug lookup by name or code ─────────────────────────────── ▸ sealed
```

**Raising the artifact** - the contribution ground rules live in
`CONTRIBUTING.md`; the Rust conventions in `AGENTS-RUST.md`. Open an
issue first to discuss a change.

**Status** - Windows installers ship from the
[release workflow](.github/workflows/windows-release.yml).

---

```
  ─────────────────────────────────────────
   A drug's peak month is not trivia.
   It is the demand curve telling the truth.
  ─────────────────────────────────────────
```

Licensed under the [MIT License](LICENSE).