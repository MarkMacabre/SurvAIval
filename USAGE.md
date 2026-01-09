# SurvAIval Usage Guide

## What is SurvAIval?

SurvAIval is an **offline-first survival knowledge system** that provides immediate, verified emergency guidance when you need it most. Unlike AI chatbots, it never makes up information—if it doesn't have verified data, it tells you explicitly.

## Key Features

- ✅ **Fully Offline** - Works without internet connection
- ✅ **No Hallucination** - Never fabricates medical advice
- ✅ **Deterministic** - Same question always gives same answer
- ✅ **Single Best Match** - One clear answer, not a confusing list
- ✅ **Verified Sources** - All guidance from trusted medical/survival manuals
- ✅ **Instant Response** - Query results in < 5ms

## Getting Started

### Step 1: Initialize Your Database

```bash
./survaival init --db survival.db
```

This creates an empty database ready to store survival knowledge.

### Step 2: Load Verified Survival Data

```bash
./survaival ingest --db survival.db --file test_data/sample_units.json
```

The sample data includes 5 verified survival units:
- Severe bleeding control
- CPR for cardiac arrest
- Choking (Heimlich maneuver)
- Fracture immobilization
- Burn treatment

### Step 3: Query for Guidance

```bash
./survaival query --db survival.db "bleeding emergency"
```

## Query Examples

### Medical Emergencies

**Severe Bleeding**
```bash
./survaival query --db survival.db "severe bleeding now" \
    --urgency "Immediate" \
    --tools "Tourniquet,Bandage"
```

**Cardiac Arrest / No Pulse**
```bash
./survaival query --db survival.db "no pulse not breathing" \
    --urgency "Immediate"
```

**Choking**
```bash
./survaival query --db survival.db "choking cant breathe" \
    --urgency "Immediate"
```

**Fracture / Broken Bone**
```bash
./survaival query --db survival.db "broken bone arm" \
    --urgency "High" \
    --tools "Splint,Bandage"
```

**Burn**
```bash
./survaival query --db survival.db "burn hot water" \
    --urgency "High" \
    --tools "Bandage"
```

### Understanding the Output

```
════════════════════════════════════════════════════════════════
  Control severe external bleeding
  Confidence: Verified | Score: 170
════════════════════════════════════════════════════════════════

▶ IMMEDIATE ACTION:
  1. Apply direct pressure to the wound with a clean cloth
  2. If bleeding soaks through, add more dressings — do not remove original
  3. If bleeding continues, apply a tourniquet proximal to wound and note time

⚠ WHAT NOT TO DO:
  • Do NOT remove embedded objects from the wound
  • Do NOT release tourniquet once applied

▶ WHY IT MATTERS:
  Use gloves if available to protect yourself

▶ NEXT STEP:
  Maintain firm, steady pressure for at least 10 minutes

▶ FALLBACK:
  Use any clean cloth, clothing, or bare hands for pressure

▶ SOURCE:
  Field Medical Guide v2.1, Chapter 4, Pages 12-15
```

**Key Sections:**
- **IMMEDIATE ACTION**: 1-3 steps to take RIGHT NOW
- **WHAT NOT TO DO**: Critical warnings to prevent harm
- **WHY IT MATTERS**: Context for why these steps are important
- **NEXT STEP**: What to do after immediate actions
- **FALLBACK**: Alternative if you don't have the ideal tools
- **SOURCE**: Where this guidance comes from (for verification)

## Command Reference

### `init` - Initialize Database

```bash
survaival init --db <database-file>
```

Creates a new SQLite database with the required schema.

**Options:**
- `--db <file>` - Path to database file (required)

**Example:**
```bash
./survaival init --db ~/.local/share/survaival/survival.db
```

### `ingest` - Load Survival Units

```bash
survaival ingest --db <database-file> --file <json-file>
```

Loads survival units from a JSON file into the database. The JSON file must contain an array of survival units with all required fields.

**Options:**
- `--db <file>` - Path to database file (required)
- `--file <json>` - Path to JSON file containing survival units (required)

**Example:**
```bash
./survaival ingest --db survival.db --file verified_units.json
```

**Output:**
```
✓ Ingestion complete
  Ingested: 5
  Failed: 0
```

### `query` - Search for Guidance

```bash
survaival query --db <database-file> <query-text> [options]
```

Searches the database for the best matching survival guidance based on your query.

**Required:**
- `--db <file>` - Path to database file
- `<query-text>` - Description of the emergency (supports natural language)

**Optional:**
- `--tools <list>` - Comma-separated list of available tools
- `--urgency <level>` - Explicitly set urgency: Immediate, High, Medium, or Low

**Available Tools:**
- Gloves
- Tourniquet
- Bandage
- CPRMask
- Splint
- Rope
- Knife
- WaterFilter
- Firestarter
- EmergencyBlanket
- Whistle
- Flashlight
- None

**Urgency Levels:**
- **Immediate** - Life-threatening, seconds to minutes (e.g., severe bleeding, cardiac arrest)
- **High** - Minutes to one hour (e.g., fracture, moderate burn)
- **Medium** - Hours (e.g., minor wounds, dehydration)
- **Low** - Days, planning/preparation (e.g., shelter building)

**Example:**
```bash
./survaival query --db survival.db "bleeding wound woods" \
    --tools "Bandage,Tourniquet" \
    --urgency "Immediate"
```

## Understanding Query Matching

### How Queries Work

SurvAIval uses **keyword matching** from dictionaries, not AI inference. This ensures:
1. **No hallucination** - It can only return data it has
2. **Deterministic** - Same query = same result
3. **Transparent** - You can see why it matched

### Query Components

Your query is analyzed for:

1. **Urgency Keywords**
   - Immediate: "now", "immediately", "urgent", "dying", "severe"
   - High: "soon", "fast", "quickly"
   - Medium: "hours", "later"
   - Low: "planning", "prepare"

2. **Scenario Keywords**
   - Bleeding: "bleed", "bleeding", "blood", "cut"
   - Cardiac Arrest: "no pulse", "not breathing", "collapse"
   - Choking: "choking", "can't breathe", "airway blocked"
   - Etc.

3. **Environment Keywords**
   - Wilderness: "woods", "forest", "trail"
   - Urban: "city", "street", "building"
   - Mountain: "alpine", "peak", "high altitude"
   - Etc.

4. **Available Tools**
   - Specified with `--tools` parameter
   - Used to find best match for your situation

### Scoring System

Each potential match gets a score based on:
- **Base**: 100 points (urgency match)
- **Tools**: 0-30 points (do you have what's needed?)
- **Confidence**: 0-40 points (how verified is this guidance?)
- **Relevance**: 0-25 points (how well does query match content?)
- **Safety**: 0-15 points (are fallbacks available?)

**Maximum score**: 210 points

The system returns the **single best match**, not a list.

### When There's No Match

If the system can't find verified guidance, you'll see:

```
✗ Query failed: I do not have verified information for this scenario.

Seek professional help now and upload verified guidance to SurvAIval for future use.
```

This is **by design**. The system will never make up an answer.

## Creating Your Own Survival Units

You can add your own verified survival guidance by creating a JSON file:

```json
[
  {
    "title": "Treat minor cut",
    "immediate_action": [
      "Wash hands thoroughly with soap and water",
      "Clean wound with clean water",
      "Apply pressure with clean cloth until bleeding stops"
    ],
    "expanded_steps": [
      "Pat dry with clean towel",
      "Apply antibiotic ointment if available",
      "Cover with sterile bandage"
    ],
    "warnings": [
      "Do NOT use dirty cloth on open wound",
      "Do NOT ignore signs of infection (redness, swelling, pus)"
    ],
    "scenario_tags": ["Bleeding"],
    "environment_tags": ["Urban", "Wilderness"],
    "urgency": "Medium",
    "required_tools": ["Bandage"],
    "no_tool_fallback": ["Use clean cloth as makeshift bandage"],
    "failure_fallback": ["Keep wound clean and monitor for infection"],
    "source_reference": "First Aid Manual 2020, Chapter 3",
    "confidence": "Verified"
  }
]
```

**Required Fields:**
- `title` - Clear, descriptive title
- `immediate_action` - 1-3 steps (array of strings)
- `expanded_steps` - Additional details (array of strings)
- `warnings` - What NOT to do (array of strings)
- `scenario_tags` - One or more scenarios (array)
- `environment_tags` - One or more environments (array)
- `urgency` - One of: Immediate, High, Medium, Low
- `required_tools` - Array of tool names (or ["None"])
- `source_reference` - Where this guidance comes from
- `confidence` - One of: Verified, High, Medium, Low

**Optional Fields:**
- `no_tool_fallback` - What to do if tools unavailable
- `failure_fallback` - What to do if primary approach fails

### Ingesting Your Data

```bash
./survaival ingest --db survival.db --file my_units.json
```

## Safety Guidelines

### ⚠️ IMPORTANT

1. **SurvAIval is NOT a replacement for professional medical care**
2. **Always seek professional help when available**
3. **Only add guidance from verified, authoritative sources**
4. **Never rely solely on any app in a true emergency**
5. **Get proper first aid training from certified instructors**

### Best Practices

- **Before the emergency**: Load your database with verified survival units
- **Test offline**: Verify the app works without internet before you need it
- **Keep backups**: Copy your survival.db file to multiple devices
- **Update regularly**: Add new verified guidance as you find it
- **Practice**: Use the app in training scenarios, not for the first time in an emergency

## Android App (Future)

The Android version will include:
- Large PANIC button for quick access
- Compass display for orientation
- Push-to-talk voice input
- Tool selection chips
- Text-to-speech for hands-free guidance
- High-contrast UI optimized for stress situations

See `IMPLEMENTATION.md` for Android development details.

## Troubleshooting

### "No match" for valid emergency

**Problem**: Query returns no results even though you have relevant data

**Solutions**:
1. Include urgency keywords: "now", "immediately", "urgent"
2. Check your scenario tags match the query terms
3. Verify data was ingested: `sqlite3 survival.db "SELECT COUNT(*) FROM survival_unit;"`

### Database file not found

**Problem**: `Error: No such file or directory`

**Solution**: Always provide full path to database:
```bash
./survaival query --db /full/path/to/survival.db "query"
```

### Ingestion fails

**Problem**: "Failed to ingest" errors

**Solutions**:
1. Validate your JSON syntax: `cat file.json | jq .`
2. Check all required fields are present
3. Verify enum values match exactly (case-sensitive)

## Getting Help

1. Read this guide thoroughly
2. Check `BUILD.md` for technical details
3. Review `IMPLEMENTATION.md` for architecture
4. Examine `test_data/sample_units.json` for examples
5. Open an issue on GitHub with:
   - Your query command
   - Expected vs actual output
   - Database contents (if relevant)

## Contributing Survival Data

If you have verified survival guidance to contribute:

1. Format it as JSON following the schema
2. Include proper source references
3. Mark confidence level honestly
4. Submit via pull request
5. Include verification of source authenticity

Remember: **Lives may depend on this data. Only verified, authoritative sources.**

## License

SurvAIval is released under the MIT License. See LICENSE file for details.

**Disclaimer**: This software is provided "as is" without warranty. The developers are not responsible for any harm resulting from the use or misuse of this software. Always seek professional medical care.
