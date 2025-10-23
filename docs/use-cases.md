# IBM 1130 Emulator - Use Cases & Test Scenarios

## Use Case 1: Assemble and Execute Sample Program (TOP PRIORITY)

**User Story:** As an assembler student, I want to select an example program, assemble it, see the output, and step through execution.

### Scenario 1.1: Load and Assemble Working Example

**Pre-conditions:**
- Server running at `http://localhost:1130`
- Browser loaded with fresh page (cache cleared)
- Default sample program visible in editor

**User Actions:**
1. Navigate to `http://localhost:1130?ts=<timestamp>`
2. Click "Assembler" tab
3. Verify sample code is visible in editor
4. Click "Assemble" button

**Expected Results:**
1. Status changes from "Ready" → "Assembling..." → "Success"
2. Assembler Output shows:
   - "✓ Assembly successful" message
   - Origin address (e.g., "Origin: 0x0100")
   - Entry Point address
   - Code Size in words
   - "Program loaded into memory and ready to execute."
3. Status bar shows:
   - Status: Success (in green)
   - Lines: 15 (or actual line count)
   - Errors: 0
   - Code Size: actual word count

**Internal Validation (Console Logs):**
```
[Assembler] Assemble button clicked
[Assembler] Code length: <N> chars
[Assembler] About to call cpu.assemble()
[Assembler] Got mutable borrow of CPU
[WASM] assemble() called
[WASM] Assembler created, calling assemble()
[WASM] Assembly successful, loading <N> words
[Assembler] Assembly call returned
[Assembler] Got Ok result from WASM
[Assembler] Deserialized result, success=true
```

**Next Steps:**
- Program should be loaded in memory
- Controls (Run, Step) should be ready for execution

**Validation Criteria:**
- [ ] All expected console logs appear in correct order
- [ ] Status updates are visible in UI
- [ ] Output message displays all required information
- [ ] No errors in console
- [ ] Status bar shows correct statistics

---

### Scenario 1.2: Assemble Program with Syntax Error

**Pre-conditions:**
- Server running
- Assembler tab open
- Editor contains code with known syntax error

**User Actions:**
1. Modify sample program to introduce error (e.g., use invalid instruction)
2. Click "Assemble" button

**Expected Results:**
1. Status changes: "Ready" → "Assembling..." → "Error"
2. Assembler Output shows:
   - "✗ Assembly failed" message
   - List of errors with line numbers
3. Status bar shows:
   - Status: Error (in red)
   - Errors: >0
   - Code Size: N/A

**Internal Validation (Console Logs):**
```
[Assembler] Assemble button clicked
[Assembler] Code length: <N> chars
[Assembler] About to call cpu.assemble()
[Assembler] Got mutable borrow of CPU
[WASM] assemble() called
[WASM] Assembler created, calling assemble()
[WASM] Assembly failed: <error message>
[Assembler] Assembly call returned
[Assembler] Got Ok result from WASM
[Assembler] Deserialized result, success=false
```

**Validation Criteria:**
- [ ] Error message is displayed in output panel
- [ ] Status shows "Error" state
- [ ] Error count is accurate
- [ ] No JavaScript exceptions

---

### Scenario 1.3: Execute Assembled Program (Step-by-step)

**Pre-conditions:**
- Program successfully assembled (Scenario 1.1 completed)
- Status shows "Success"
- Entry point known

**User Actions:**
1. Click "Step" button
2. Observe register updates
3. Click "Step" again
4. Continue until program completes

**Expected Results:**
1. Each step:
   - IAR (Instruction Address Register) updates
   - ACC, EXT, or index registers update as appropriate
   - Instruction count increments
   - Memory viewer highlights current instruction
2. Registers panel shows updated values
3. Console panel shows execution trace

**Internal Validation (Console Logs):**
```
[CPU] Stepping instruction at 0x<addr>
[CPU] Decoded: <instruction>
[CPU] ACC: 0x<old> → 0x<new>
[CPU] IAR: 0x<old> → 0x<new>
```

**Validation Criteria:**
- [ ] Registers update correctly
- [ ] IAR follows program flow
- [ ] Memory contents reflect execution
- [ ] Step count increments

---

### Scenario 1.4: Run Program Continuously

**Pre-conditions:**
- Program successfully assembled
- Status shows "Success"

**User Actions:**
1. Click "Run" button
2. Observe execution
3. Click "Reset" button

**Expected Results:**
1. Program executes until WAIT or halt
2. Registers update during execution
3. Final state is displayed
4. Reset returns CPU to initial state

**Validation Criteria:**
- [ ] Program runs to completion
- [ ] Final register values are correct
- [ ] Reset clears all registers
- [ ] Memory returns to initial state

---

## Use Case 2: Editor Operations

### Scenario 2.1: Clear Editor

**Pre-conditions:**
- Editor contains text

**User Actions:**
1. Click "Clear" button

**Expected Results:**
1. Editor text is cleared
2. Status returns to "Ready"
3. Output shows "Ready to assemble..."
4. Error count resets to 0

**Internal Validation:**
```
[Assembler] Clear button clicked
```

**Validation Criteria:**
- [ ] Editor is empty
- [ ] Status reset
- [ ] Statistics cleared

---

### Scenario 2.2: Edit Code

**Pre-conditions:**
- Editor contains sample code

**User Actions:**
1. Click in editor
2. Modify code
3. Observe line count update

**Expected Results:**
1. Code updates as typed
2. Line count updates in status bar
3. No automatic assembly triggered

**Validation Criteria:**
- [ ] Editor accepts input
- [ ] Line count is accurate
- [ ] No errors during editing

---

## Use Case 3: Memory Inspection

### Scenario 3.1: View Memory Contents

**Pre-conditions:**
- Program assembled and loaded

**User Actions:**
1. Click "Memory" tab
2. Scroll through memory view
3. Click on memory address

**Expected Results:**
1. Memory view shows addresses and contents
2. Loaded program visible at origin
3. Values display in hexadecimal

**Validation Criteria:**
- [ ] Memory addresses are sequential
- [ ] Program code matches assembly output
- [ ] Data values are correct

---

## Use Case 4: Register Inspection

### Scenario 4.1: View CPU State

**Pre-conditions:**
- CPU initialized

**User Actions:**
1. Click "Registers" tab
2. Observe initial state
3. Execute instruction
4. Observe updated state

**Expected Results:**
1. All registers display with values
2. Carry and Overflow flags visible
3. Instruction count shown
4. Values update after each step

**Validation Criteria:**
- [ ] All registers present
- [ ] Values formatted correctly (hex)
- [ ] Updates occur after execution

---

## Use Case 5: Console Panel

### Scenario 5.1: View System Messages

**Pre-conditions:**
- Application loaded

**User Actions:**
1. Click "Console Panel" tab
2. Perform various operations
3. Observe messages

**Expected Results:**
1. System messages appear in chronological order
2. Different message types visible (info, warning, error)
3. Messages provide useful debugging information

**Validation Criteria:**
- [ ] Messages are readable
- [ ] Timestamps present (if applicable)
- [ ] No garbled text

---

## Use Case 6: I/O Devices

### Scenario 6.1: View Device Status

**Pre-conditions:**
- Application loaded

**User Actions:**
1. Click "I/O Devices" tab
2. View available devices

**Expected Results:**
1. Device list shows:
   - Keyboard (status: Ready)
   - Printer (status: Ready)
   - Other configured devices
2. Each device shows current state

**Validation Criteria:**
- [ ] All devices listed
- [ ] Status indicators correct
- [ ] Future: Device operations work

---

## Test Execution Priority

1. **P0 (Critical):** Scenario 1.1, 1.2 - Assembly must work
2. **P1 (High):** Scenario 1.3, 1.4 - Execution must work
3. **P2 (Medium):** Use Cases 2, 3, 4 - Inspection/editing
4. **P3 (Low):** Use Cases 5, 6 - Secondary features

---

## Automated Test Template

For each scenario, Playwright tests should:

```javascript
// 1. Setup
- Navigate with cache-bust timestamp
- Clear console logs
- Verify initial state

// 2. Execute
- Perform user actions
- Wait for UI updates
- Capture console logs

// 3. Validate
- Check UI elements
- Verify console log sequence
- Assert expected values
- Take screenshot

// 4. Cleanup
- Log results
- Save artifacts if failed
```

---

## Known Issues to Test

1. **WASM Caching:** Verify fresh code loads
2. **State Updates:** Ensure UI reflects backend changes
3. **Error Handling:** Proper error display
4. **Deserialization:** Result parsing works correctly

---

## Sample Code for Testing

### Working Program (Simple Loop)
```assembly
*
* Simple Counter Test
*
    ORG  /100
LOOP LDX 1 COUNT
     LD  A
     A   ONE
     STO A
     MDX 1 -1
     BNZ LOOP
     WAIT

A    DC  0
ONE  DC  1
COUNT DC  5
     END LOOP
```

### Program with Error (for testing error handling)
```assembly
*
* Syntax Error Test
*
    ORG  /100
START INVALID X 123    ; Invalid instruction
      LD  A
      WAIT
A DC 0
     END START
```
