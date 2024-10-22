const hook = require('.');

// Start the listener with a custom inactivity duration (e.g., 30 seconds)
hook.start(1 * 10); // Pass the number of seconds after which the user is considered inactive

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