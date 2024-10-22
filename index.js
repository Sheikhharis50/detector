const hook = require('.');  // The compiled Neon addon

// Start the listener
hook.start();

// Function to check if the user is active
function checkUserActivity() {
    const isActive = hook.isActive();
    console.log("User active:", isActive);

    // Check every 1 seconds
    setTimeout(checkUserActivity, 1000);  // Adjust delay as needed
}

// Start checking user activity in an infinite loop
checkUserActivity();

// Optional: Stop listener after some time (e.g., 5 minutes)
setTimeout(() => {
    hook.stop();
    console.log("Stopped listener.");
}, 20 * 60 * 1000);  // Stop after 20 minutes