// Quick manual test for AnkiDroid integration
// This script will test the main operations

const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);

async function testAnkiDroidIntegration() {
    console.log('🧪 Testing AnkiDroid Integration...\n');
    
    try {
        // Test hello command first
        console.log('1. Testing hello command...');
        const helloResult = await execAsync('adb shell am start -n com.tauri.ankidroid.demo/.MainActivity');
        console.log('   App started:', helloResult.stdout.trim());
        
        // Give app time to load
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // Check app logs for any errors
        console.log('\n2. Checking app logs...');
        const logResult = await execAsync('adb logcat -d | grep -E "(tauri|ankidroid|AnkiDroid|ERROR|FATAL)" | tail -10');
        console.log('   Recent logs:', logResult.stdout || 'No relevant logs found');
        
        // Check if AnkiDroid is accessible
        console.log('\n3. Checking AnkiDroid accessibility...');
        const ankidroidCheck = await execAsync('adb shell content query --uri content://com.ichi2.anki.flashcards/notes');
        console.log('   AnkiDroid query result:', ankidroidCheck.stdout.trim() || 'Query successful');
        
        console.log('\n✅ Basic integration test completed');
        
    } catch (error) {
        console.error('❌ Test failed:', error.message);
        if (error.stdout) console.log('stdout:', error.stdout);
        if (error.stderr) console.log('stderr:', error.stderr);
    }
}

testAnkiDroidIntegration();