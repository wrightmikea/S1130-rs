# IBM 1130 Assembler Conventions Research

**Date:** 2025-10-23
**Purpose:** Document IBM 1130 assembler syntax conventions to guide assembler implementation

---

## Executive Summary

Research into IBM 1130 assembler conventions reveals:
- **Column-based format** (punch card era heritage)
- **Hexadecimal notation** using `/` prefix (e.g., `/3C00`)
- **Flexible modern format** using tabs in cross-assemblers
- **Label positioning** determines whether first token is label or operation

---

## Sources

### Primary References
- **IBM 1130 Assembler Language Manual**: Order No. GC26-5927-2 (1966)
  - URL: http://media.ibm1130.org/E0023.pdf (certificate issue prevented direct access)
  - Alternate: https://bitsavers.org/pdf/ibm/1130/lang/C26-5927-2_1130_Assembler_Language_1966.pdf

### Secondary References
- **SIMH IBM 1130 Emulator**: https://github.com/simh/simh/tree/master/Ibm1130
  - Cross-assembler (asm1130) implementation
  - Assembly code examples in issues and test files
- **IBM1130.net**: https://ibm1130.net/functional/
  - Functional characteristics documentation
  - CPU instruction reference

### Code Examples
- **SIMH Issue #166**: IBM 2250 Display test code
  - URL: https://github.com/simh/simh/issues/166
  - Shows real-world assembly formatting

---

## Column Format (Punch Card Heritage)

### Historical Context
IBM 1130 assembly was designed for 80-column punch cards:
- **Columns 1-20**: Reserved for assembler output (location counters, machine code)
- **Columns 21-72**: Source code (label, operation, operand, comments)
- **Columns 73-80**: Ignored (typically used for card sequence numbers)

### Field Breakdown
Within columns 21-72:
- **Label field**: Starts at column 21
- **Operation field**: Starts around column 27
- **Operand field**: Follows operation
- **Comment field**: Can follow operand (often preceded by `*`)

### Practical Impact
The 20-column offset meant that if source changes were needed, programmers would duplicate cards to get a deck with columns 1-20 blank for the next assembly pass. The assembler would punch code into the start of the card just read during the second pass.

---

## Modern Tab-Delimited Format

The SIMH cross-assembler (asm1130) accepts a more flexible format:

```
label<TAB>opcode<TAB>flags<TAB>operand
```

**Advantages:**
- Easier to type in modern editors
- No need to count columns
- Tab-separated fields are clearly delimited

**Compatibility:**
- SIMH assembler auto-detects format based on presence of tabs
- If tabs present: use tab format
- If no tabs: strict column layout

---

## Label Recognition

### Key Principle
**How to distinguish labels from operations:**

1. **Fixed column format**: Label starts in column 21, operation in column 27
2. **Tab format**: `label<TAB>operation<TAB>...`
3. **Whitespace heuristic** (for modern free-format):
   - Line starts with whitespace → NO label, first token is operation
   - Line starts with non-whitespace → first token is LABEL, second is operation

### Examples

```assembly
        LD   A        * No label - starts with spaces
START   LD   A        * Label "START", operation "LD"
A       DC   100      * Label "A", pseudo-op "DC"
```

### Problem Case
Single-letter labels that match instruction mnemonics:

```assembly
A       DC   100      * Label "A" (not instruction A/Add)
        A    B        * Instruction "A" (Add), operand "B"
```

**Solution**: Use whitespace or column position to disambiguate.

---

## Number Formats

### Hexadecimal Notation - `/` Prefix

From SIMH code examples (Issue #166):

```assembly
CONBF DC /3C00        * Hexadecimal constant 0x3C00
LDATA DC /0101        * Hexadecimal constant 0x0101
SENS3 DC /BF68        * Hexadecimal constant 0xBF68
LAMPS DC /CF01        * Hexadecimal constant 0xCF01
```

**Syntax**: `/` followed by hexadecimal digits
**Range**: 4 hex digits for 16-bit words (/0000 to /FFFF)

### Decimal Notation

Plain numbers without prefix:

```assembly
COUNT DC 100          * Decimal 100
SIZE  DC 42           * Decimal 42
```

### Octal Notation

**Traditional IBM style**: Leading zero (C-style)

```assembly
VALUE DC 0777         * Octal 777 (511 decimal)
ADDR  DC 0100         * Octal 100 (64 decimal)
```

**Note**: Not confirmed whether IBM 1130 also used `/` for octal in addition to hex. Current evidence strongly suggests `/` = hexadecimal only.

### Alternative Number Formats

Some IBM assemblers supported:
- **Hex with X prefix**: `X'3C00'` or `X3C00`
- **Binary**: `B'1100'` or similar
- **Character constants**: `C'A'`

**Status**: Need to verify if IBM 1130 supported these alternative formats.

---

## Pseudo-Operations (Pseudo-Ops)

### ORG - Set Origin Address

```assembly
        ORG  /100     * Set origin to hex 0x0100
        ORG  0400     * Set origin to decimal 400
```

**Purpose**: Sets the loading address for subsequent code/data.

### DC - Define Constant

```assembly
FIVE    DC   5        * Define constant 5 (decimal)
MASK    DC   /FFFF    * Define constant 0xFFFF (hex)
ADDR    DC   START    * Define constant = address of START label
```

**Purpose**: Allocates one word and initializes it with a value.

### BSS - Block Started by Symbol

```assembly
BUFFER  BSS  100      * Reserve 100 words uninitialized
TEMP    BSS  1        * Reserve 1 word
```

**Purpose**: Reserves space without initialization (for variables).

### END - End of Assembly

```assembly
        END           * End assembly, no entry point
        END  START    * End assembly, entry point at START
```

**Purpose**: Marks end of source, optionally specifies entry point.

### EQU - Equate Symbol

```assembly
IOBASE  EQU  /F000    * Define IOBASE = 0xF000
MAXLEN  EQU  256      * Define MAXLEN = 256
```

**Purpose**: Defines a symbol as equivalent to a value (no storage allocated).

---

## Addressing Modes

### Direct Addressing

```assembly
        LD   COUNT    * Load from address COUNT
        STO  RESULT   * Store to address RESULT
```

### Indirect Addressing

**Syntax**: Operand prefixed with `/` or `*`

```assembly
        LD   /ADDR    * Load indirectly through ADDR
        STO  *PTR     * Store indirectly through PTR
```

**Conflict with Hex Constants**: This creates ambiguity:
- `/100` could mean: indirect address 100, OR hex constant 0x0100
- Context matters: in operand field vs. DC operand

**Resolution Needed**: Review actual IBM 1130 syntax to clarify.

### Indexed Addressing

**Format**: `address,index_register`

```assembly
        LD   TABLE,1  * Load from TABLE + XR1
        STO  BUFF,2   * Store to BUFF + XR2
```

**Index Registers**: 1, 2, or 3 (corresponding to XR1, XR2, XR3)

### Index Instructions (LDX, STX, MDX)

**Reversed format**: `index,address` (not `address,index`)

```assembly
        LDX  1,COUNT  * Load XR1 from COUNT
        STX  2,SAVE   * Store XR2 to SAVE
        MDX  1,DELTA  * Modify XR1 by DELTA
```

---

## Comments

### Full-Line Comments

```assembly
* This is a full-line comment
*
* Multiple comment lines
```

**Format**: Asterisk `*` in column 21 (or first non-whitespace position)

### Inline Comments

```assembly
        LD   A        * Load accumulator from A
START   XIO  DEVICE   * Initialize device
```

**Format**: Asterisk `*` after operand field

---

## Example Assembly Program

From SIMH examples and historical code:

```assembly
*
* Sample IBM 1130 Program
* Demonstrates common patterns
*
        ORG  /0100         * Start at hex address 0x0100

START   LDX  1,COUNT       * Load loop counter into XR1
        LD   ZERO          * Clear accumulator

LOOP    A    TABLE,1       * Add TABLE[XR1] to accumulator
        MDX  1,-1          * Decrement XR1
        BSC  L,LOOP,1      * Branch if XR1 >= 0

        STO  RESULT        * Store result
        WAIT               * Halt

* Data area
COUNT   DC   10            * Loop count (decimal)
ZERO    DC   0             * Constant zero
RESULT  DC   0             * Result storage
TABLE   DC   1             * Table of values
        DC   2
        DC   3
        DC   4
        DC   5
        DC   6
        DC   7
        DC   8
        DC   9
        DC   10

        END  START         * Entry point is START
```

---

## Implementation Notes for S1130-rs

### Issue 1: Hexadecimal Number Parsing

**Current behavior**: `/100` treated as indirect addressing operator
**Expected behavior**: `/100` should parse as hexadecimal 0x0100

**Fix needed in**: `crates/s1130-core/src/assembler/mod.rs:parse_expression()`

**Approach**:
1. Check if expression starts with `/` followed by hex digits
2. Parse as hex: `u16::from_str_radix(&expr[1..], 16)`
3. Fall back to symbol lookup if not valid hex

### Issue 2: Label vs. Operation Disambiguation

**Current behavior**: Parser trims line first, losing position information
**Expected behavior**: Use leading whitespace to determine if first token is label

**Fix needed in**: `crates/s1130-core/src/assembler/parser.rs:parse_line()`

**Approach**:
1. **Don't trim** the line initially
2. Check if line starts with whitespace
3. If yes → no label, parse first token as operation
4. If no → first token is label, second token is operation
5. After parsing fields, trim individual tokens

### Issue 3: Indirect Addressing Conflict

**Problem**: `/` used for both hex constants and indirect addressing

**Questions to resolve**:
- Is `*` the only indirect addressing operator?
- Is `/` used for indirect addressing at all?
- Context-dependent parsing needed?

**Recommendation**: Research more examples to clarify the conflict.

---

## Open Questions

1. **Octal format**: Does IBM 1130 support octal? If so, what prefix?
   - Leading zero (C-style): `0777`
   - Or something else?

2. **Indirect addressing**: What are the exact operators?
   - Is it `*` only?
   - Is `/` used for indirect, or only for hex constants?
   - Can both be used in different contexts?

3. **Alternative number formats**: Does IBM 1130 support?
   - `X'3C00'` style hex constants
   - `B'1010'` style binary constants
   - `C'A'` style character constants

4. **Strict column enforcement**: Should modern assembler enforce columns 21-72?
   - Or accept free-format with whitespace heuristics?
   - SIMH uses both - should we?

5. **Case sensitivity**: Are labels/opcodes case-sensitive?
   - Current impl converts to uppercase
   - Is this correct?

---

## References for Further Research

### Manuals (if accessible)
- IBM 1130 Assembler Language (GC26-5927-2)
- IBM 1130 Functional Characteristics (A26-5881-2)
- IBM 1130/1800 Macro Assembler Programming

### Online Resources
- http://ibm1130.org - IBM 1130 preservation project
- http://bitsavers.org - Scanned IBM documentation
- https://github.com/simh/simh - SIMH emulator source

### Example Code Sources
- SIMH repository test files
- DMS (Disk Monitor System) source code
- Historical program listings

---

## Revision History

- **2025-10-23**: Initial research compilation
  - Documented column format
  - Identified `/` as hexadecimal prefix
  - Clarified label recognition issue
  - Listed open questions

---

## Next Steps

1. **Implement hexadecimal `/NNNN` parsing**
2. **Fix column/whitespace-based label detection**
3. **Test with SIMH example code**
4. **Resolve indirect addressing syntax**
5. **Validate with more historical code examples**
