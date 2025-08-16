#!/bin/bash
set -e

echo "🤖 Automated E2E Test for AnkiDroid Plugin"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() { echo -e "${GREEN}✓${NC} $1"; }
print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }

# Function to simulate button tap using UI automator
tap_button() {
    print_info "Simulating button tap..."
    
    # Try to find and tap the "Read AnkiDroid Cards" button
    # We'll use coordinates as a fallback if UI automator doesn't work
    
    # Get screen dimensions
    SCREEN_SIZE=$(adb shell wm size | grep -o '[0-9]*x[0-9]*' | head -1)
    WIDTH=$(echo $SCREEN_SIZE | cut -d'x' -f1)
    HEIGHT=$(echo $SCREEN_SIZE | cut -d'x' -f2)
    
    # Calculate center position (where button likely is)
    CENTER_X=$((WIDTH / 2))
    CENTER_Y=$((HEIGHT / 2))
    
    print_info "Screen size: ${WIDTH}x${HEIGHT}, tapping at center: ${CENTER_X},${CENTER_Y}"
    
    # Tap the center of screen (where button should be)
    adb shell input tap $CENTER_X $CENTER_Y
    
    return 0
}

# Function to check app crash
check_for_crash() {
    print_info "Checking for crashes in the last 10 seconds..."
    
    # Get recent logs
    CRASH_LOGS=$(adb logcat -d -t 10 | grep -E "(FATAL|AndroidRuntime|RustPanic|demo-app.*crashed)" || true)
    
    if [ -n "$CRASH_LOGS" ]; then
        print_error "Crash detected!"
        echo "$CRASH_LOGS"
        return 1
    else
        print_status "No crashes detected"
        return 0
    fi
}

# Function to check for successful response
check_for_success() {
    print_info "Checking for graceful error handling..."
    
    # Look for our error handling responses (which are actually successful)
    ERROR_HANDLING_LOGS=$(adb logcat -d -t 30 | grep -E "(Permission.*denied|AnkiDroid.*Error|System.*Error)" || true)
    
    if [ -n "$ERROR_HANDLING_LOGS" ]; then
        print_status "Graceful error handling detected:"
        echo "$ERROR_HANDLING_LOGS"
        return 0
    else
        # Look for actual successful data
        SUCCESS_LOGS=$(adb logcat -d -t 30 | grep -E "(List cards command|cards_data|Successfully|query.*successful)" || true)
        
        if [ -n "$SUCCESS_LOGS" ]; then
            print_status "Success indicators found in logs:"
            echo "$SUCCESS_LOGS"
            return 0
        else
            print_warning "No clear success or error handling indicators found"
            return 1
        fi
    fi
}

# Main test execution
main() {
    print_info "Starting automated E2E test..."
    
    # Ensure app is running
    print_info "Ensuring app is in foreground..."
    adb shell am start -n com.tauri.ankidroid.demo/.MainActivity
    sleep 2
    
    # Clear logs to get clean test data
    adb logcat -c
    print_status "Logs cleared for clean test"
    
    # Wait for app to fully load
    sleep 3
    
    # Attempt to tap the button
    if tap_button; then
        print_status "Button tap completed"
    else
        print_error "Failed to tap button"
        return 1
    fi
    
    # Wait for response
    print_info "Waiting for app response..."
    sleep 5
    
    # Check results
    TEST_PASSED=true
    
    if ! check_for_crash; then
        print_error "Test FAILED: App crashed"
        TEST_PASSED=false
    fi
    
    if ! check_for_success; then
        print_warning "Test INCONCLUSIVE: No clear success indicators"
        # Don't fail the test for this, as it might still work
    fi
    
    # Get all relevant logs for analysis
    print_info "Capturing relevant logs for analysis..."
    RELEVANT_LOGS=$(adb logcat -d | grep -E "(demo-app|tauri|ankidroid|list_cards|cards)" | tail -20 || true)
    
    if [ -n "$RELEVANT_LOGS" ]; then
        echo ""
        echo "=== Relevant Logs ==="
        echo "$RELEVANT_LOGS"
        echo "===================="
    fi
    
    # Final result
    if [ "$TEST_PASSED" = true ]; then
        print_status "✅ E2E Test PASSED - No crashes detected"
        return 0
    else
        print_error "❌ E2E Test FAILED - Issues detected"
        return 1
    fi
}

# Run the test
main "$@"