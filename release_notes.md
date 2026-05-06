# Crest v0.2.5 — The Visual Revolution
 
This release introduces a ground-up UI overhaul, migrating the launcher's aesthetic to the high-fidelity design generated via Google Stitch.

### 🏔️ Highlights:
- **Stitch High-Fidelity UI**: A complete visual redesign matching the "Web Page Replicator" mockup. Deep `#12121e` backgrounds, refined glassy borders, and indigo accents.
- **50/50 Split Layout**: The Command Palette now features a perfectly balanced 50/50 split, giving the preview panel a dedicated, high-impact space.
- **Minimalist Search Header**: Removed all visual clutter from the search bar. Larger typography and borderless inputs provide a cleaner, more focused experience.
- **Refined Detail View**: A revamped preview panel with left-aligned hero icons, descriptive metadata blocks, and an interactive HUD-style shortcut hint box.
- **Enhanced Visual Hierarchy**: Simplified selection highlights and status markers (Orange for Apps, Blue for Files) for instant visual recognition.

### 🛠️ UI Changes:
- Updated `index.css` with the "Technical Minimalism" color tokens.
- Refactored `PreviewPanel.tsx` and `PreviewPanel.css` for the new hero/body structure.
- Streamlined `SearchInput.css` and `ResultList.css` to match the minimalist header and selection styles.
- Balanced the split-view layout in `CommandPalette.css`.
