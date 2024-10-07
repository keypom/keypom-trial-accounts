// index.ts

/**
 * Main entry point for the Trial Accounts package.
 *
 * This package provides functions to deploy trial contracts,
 * create trials, add trial accounts, activate trial accounts,
 * perform actions, and broadcast transactions.
 *
 * @packageDocumentation
 */

export { deployTrialContract } from "./createContract";
export { createTrial } from "./createTrial";
export { addTrialAccounts } from "./addTrialKeys";
export { activateTrialAccounts } from "./activateTrial";
export { performActions, broadcastTransaction } from "./performAction";
export { initNear } from "./utils";

// Export types for user convenience
export * from "./types";