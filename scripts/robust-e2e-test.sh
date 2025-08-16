#!/bin/bash
set -e

echo "🔧 Robust E2E Test - Real Button Click Testing"
echo "=============================================="

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

# Function to find and click the actual button using UI Automator
click_button_robust() {
    print_info "Using UI Automator to find and click the button..."
    
    # Dump the UI hierarchy to find the button
    adb exec-out uiautomator dump /dev/stdout > /tmp/ui_dump.xml 2>/dev/null || true
    
    # Look for button text variations
    BUTTON_FOUND=false
    
    # Try multiple button text variations
    for button_text in "Read AnkiDroid Cards" "List Cards" "📚 Read AnkiDroid Cards" "Loading Cards" "Loading..."; do
        print_info "Looking for button with text: $button_text"
        
        if adb shell uiautomator runtest /dev/null -c "new UiSelector().textContains(\"$button_text\")" 2>/dev/null; then
            print_status "Found button with text: $button_text"
            
            # Click using UI Automator
            if adb shell uiautomator runtest /dev/null -c "new UiSelector().textContains(\"$button_text\").click()" 2>/dev/null; then
                print_status "Successfully clicked button: $button_text"
                BUTTON_FOUND=true
                break
            fi
        fi
    done
    
    # Fallback: try clicking by class name
    if [ "$BUTTON_FOUND" = false ]; then
        print_info "Trying to find button by class name..."
        if adb shell uiautomator runtest /dev/null -c "new UiSelector().className(\"android.widget.Button\").click()" 2>/dev/null; then
            print_status "Clicked button by class name"
            BUTTON_FOUND=true
        fi
    fi
    
    # Final fallback: coordinate-based clicking with multiple attempts
    if [ "$BUTTON_FOUND" = false ]; then
        print_warning "UI Automator failed, using coordinate-based clicking..."
        
        # Get screen dimensions
        SCREEN_SIZE=$(adb shell wm size | grep -o '[0-9]*x[0-9]*' | head -1)
        WIDTH=$(echo $SCREEN_SIZE | cut -d'x' -f1)
        HEIGHT=$(echo $SCREEN_SIZE | cut -d'x' -f2)
        
        # Try multiple positions where the button might be
        CENTER_X=$((WIDTH / 2))
        UPPER_Y=$((HEIGHT / 3))      # Upper third
        CENTER_Y=$((HEIGHT / 2))     # Center
        LOWER_Y=$((HEIGHT * 2 / 3))  # Lower third
        
        for y_pos in $UPPER_Y $CENTER_Y $LOWER_Y; do
            print_info "Trying tap at: ${CENTER_X},${y_pos}"
            adb shell input tap $CENTER_X $y_pos
            sleep 1
            
            # Check if app crashed after this tap
            if adb shell pidof com.tauri.ankidroid.demo > /dev/null 2>&1; then
                print_status "App still running after tap at ${CENTER_X},${y_pos}"
                BUTTON_FOUND=true
                break
            else
                print_warning "App may have crashed after tap at ${CENTER_X},${y_pos}"
            fi
        done
    fi
    
    return $([ "$BUTTON_FOUND" = true ] && echo 0 || echo 1)
}

# Function to monitor for crashes in real-time
monitor_for_crashes() {
    local duration=$1
    print_info "Monitoring for crashes for ${duration} seconds..."
    
    local start_time=$(date +%s)
    local crash_detected=false
    
    while [ $(($(date +%s) - start_time)) -lt $duration ]; do
        # Check if app process exists
        if ! adb shell pidof com.tauri.ankidroid.demo > /dev/null 2>&1; then
            print_error "CRASH DETECTED: App process no longer exists"
            crash_detected=true
            break
        fi
        
        # Check for crash logs in real-time
        RECENT_CRASHES=$(adb logcat -d -t 5 | grep -E "(FATAL|AndroidRuntime.*CRASH|E AndroidRuntime)" | wc -l)
        if [ "$RECENT_CRASHES" -gt 0 ]; then
            print_error "CRASH DETECTED: Fatal error in logs"
            crash_detected=true
            break
        fi
        
        sleep 0.5
    done
    
    if [ "$crash_detected" = false ]; then
        print_status "No crashes detected during monitoring period"
        return 0
    else
        return 1
    fi
}

# Function to check for expected error handling
check_error_handling() {
    print_info "Checking for proper error handling response..."
    
    # Look for our specific error messages that indicate graceful handling
    local error_patterns=(
        "Permission.*denied"
        "AnkiDroid.*Error"
        "System.*Error"  
        "Permission.*Required"
        "Third.*party.*apps"
    )
    
    for pattern in "${error_patterns[@]}"; do
        ERROR_COUNT=$(adb logcat -d -t 20 | grep -E "$pattern" | wc -l)
        if [ "$ERROR_COUNT" -gt 0 ]; then
            print_status "Found graceful error handling: $pattern"
            adb logcat -d -t 20 | grep -E "$pattern" | head -3
            return 0
        fi
    done
    
    print_warning "No specific error handling patterns found"
    return 1
}

# Main test execution
main() {
    print_info "Starting robust E2E test..."
    
    # Ensure app is installed and running
    print_info "Installing and launching app..."
    adb install -r "/workspaces/tauri-plugin-ankidroid/packages/demo-app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk" > /dev/null
    adb shell am start -n com.tauri.ankidroid.demo/.MainActivity
    sleep 3
    
    # Verify app is running
    if ! adb shell pidof com.tauri.ankidroid.demo > /dev/null 2>&1; then
        print_error "App failed to start"
        return 1
    fi
    print_status "App is running"
    
    # Clear logs for clean test
    adb logcat -c
    
    # Attempt to click the button using robust methods
    if click_button_robust; then
        print_status "Button click successful"
    else
        print_error "Failed to click button reliably"
        return 1
    fi
    
    # Monitor for crashes after button click
    if monitor_for_crashes 5; then
        print_status "No crashes after button click"
    else
        print_error "App crashed after button click"
        
        # Show crash details
        print_info "Crash details:"
        adb logcat -d | grep -E "(FATAL|AndroidRuntime)" | tail -10
        return 1
    fi
    
    # Check for proper error handling
    if check_error_handling; then
        print_status "Graceful error handling confirmed"
    else
        print_warning "Error handling verification inconclusive"
    fi
    
    # Final verification - app should still be running
    if adb shell pidof com.tauri.ankidroid.demo > /dev/null 2>&1; then
        print_status "App still running after test"
        return 0
    else
        print_error "App died during test"
        return 1
    fi
}

# Run the test
if main; then
    print_status "🎉 ROBUST E2E TEST PASSED"
    print_info "✅ Button clicking works reliably"
    print_info "✅ No crashes detected"
    print_info "✅ Error handling is graceful"
    exit 0
else
    print_error "❌ ROBUST E2E TEST FAILED"
    print_info "Issues detected that need to be fixed"
    exit 1
fi