const userActivity = require('.');

// Start the user activity listener
userActivity.startListener();

// Periodically check if the user is active
setInterval(() => {
    const isActive = userActivity.isUserActive();
    console.log(`User is active: ${isActive}`);
}, 2000);  // Check every 2 seconds

// Stop the listener after some time (e.g., after 30 seconds)
setTimeout(() => {
    userActivity.stopListener();
    console.log('Stopped listening for user activity.');
}, 30000);  // Stop after 30 seconds