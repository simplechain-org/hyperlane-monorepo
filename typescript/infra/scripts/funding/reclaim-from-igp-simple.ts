import 'dotenv/config';
import { ethers } from 'ethers';
import { formatEther, parseEther } from 'ethers/lib/utils.js';
import { rootLogger } from '@hyperlane-xyz/utils';

// IGP ABI - 只需要 claim 函数
const IGP_ABI = [
  'function claim() external',
  'function beneficiary() external view returns (address)',
] as const;

interface ChainConfig {
  chainName: string;
  rpcUrl: string;
  igpAddress: string;
  relayerAddress: string;
  igpBalanceThreshold: string; // ETH amount, e.g., "0.1"
  relayerBalanceThreshold: string; // ETH amount, e.g., "0.5"
  privateKey?: string; // Optional: if not provided, will use env var or prompt
}

/**
 * Parse chain configurations from environment variables
 * Expected format:
 * CHAINS=chain1,chain2,chain3
 * RELAYER_ADDRESS=0x... (global, shared across all chains)
 * CHAIN1_RPC_URL=https://...
 * CHAIN1_IGP_ADDRESS=0x...
 * CHAIN1_IGP_THRESHOLD=0.1 (optional, default 0.1)
 * CHAIN1_RELAYER_THRESHOLD=0.5 (optional, default 0.5, per-chain because token prices differ)
 */
function parseChainConfigs(): ChainConfig[] {
  const chainsEnv = process.env.CHAINS;
  if (!chainsEnv) {
    throw new Error('CHAINS environment variable is required');
  }

  // Global relayer address (shared across all chains)
  const relayerAddress = process.env.RELAYER_ADDRESS;
  if (!relayerAddress) {
    throw new Error('RELAYER_ADDRESS environment variable is required');
  }

  const chainNames = chainsEnv.split(',').map((c) => c.trim().toUpperCase());
  const configs: ChainConfig[] = [];

  for (const chainName of chainNames) {
    const rpcUrl = process.env[`${chainName}_RPC_URL`];
    const igpAddress = process.env[`${chainName}_IGP_ADDRESS`];
    const igpThreshold = process.env[`${chainName}_IGP_THRESHOLD`] || '0.1';
    // Relayer threshold is per-chain because token prices differ across chains
    const relayerThreshold =
      process.env[`${chainName}_RELAYER_THRESHOLD`] || '0.5';
    const privateKey =
      process.env[`${chainName}_PRIVATE_KEY`] || process.env.PRIVATE_KEY;

    if (!rpcUrl || !igpAddress) {
      rootLogger.warn(
        `Skipping ${chainName}: missing required config (RPC_URL or IGP_ADDRESS)`,
      );
      continue;
    }

    configs.push({
      chainName: chainName.toLowerCase(),
      rpcUrl,
      igpAddress,
      relayerAddress, // Use global relayer address
      igpBalanceThreshold: igpThreshold,
      relayerBalanceThreshold: relayerThreshold, // Per-chain threshold
      privateKey,
    });
  }

  return configs;
}

/**
 * Check if claim should be triggered for a chain
 */
async function shouldClaim(config: ChainConfig): Promise<{
  shouldClaim: boolean;
  reason: string;
  igpBalance: string;
  relayerBalance: string;
}> {
  const provider = new ethers.providers.JsonRpcProvider(config.rpcUrl);
  const igpContract = new ethers.Contract(
    config.igpAddress,
    IGP_ABI,
    provider,
  );

  // Get balances
  const igpBalance = await provider.getBalance(config.igpAddress);
  const relayerBalance = await provider.getBalance(config.relayerAddress);

  const igpBalanceEth = formatEther(igpBalance);
  const relayerBalanceEth = formatEther(relayerBalance);

  const igpThresholdWei = parseEther(config.igpBalanceThreshold);
  const relayerThresholdWei = parseEther(config.relayerBalanceThreshold);

  // Check conditions
  if (igpBalance.gte(igpThresholdWei)) {
    return {
      shouldClaim: true,
      reason: `IGP balance (${igpBalanceEth} ETH) >= threshold (${config.igpBalanceThreshold} ETH)`,
      igpBalance: igpBalanceEth,
      relayerBalance: relayerBalanceEth,
    };
  }

  if (relayerBalance.lt(relayerThresholdWei)) {
    return {
      shouldClaim: true,
      reason: `Relayer balance (${relayerBalanceEth} ETH) < threshold (${config.relayerBalanceThreshold} ETH)`,
      igpBalance: igpBalanceEth,
      relayerBalance: relayerBalanceEth,
    };
  }

  return {
    shouldClaim: false,
    reason: `No trigger conditions met. IGP: ${igpBalanceEth} ETH, Relayer: ${relayerBalanceEth} ETH`,
    igpBalance: igpBalanceEth,
    relayerBalance: relayerBalanceEth,
  };
}

/**
 * Execute claim transaction
 */
async function executeClaim(config: ChainConfig): Promise<string> {
  if (!config.privateKey) {
    throw new Error(
      `Private key not found for ${config.chainName}. Set ${config.chainName.toUpperCase()}_PRIVATE_KEY or PRIVATE_KEY env var`,
    );
  }

  const provider = new ethers.providers.JsonRpcProvider(config.rpcUrl);
  const wallet = new ethers.Wallet(config.privateKey, provider);
  const igpContract = new ethers.Contract(
    config.igpAddress,
    IGP_ABI,
    wallet,
  );

  rootLogger.info(`Executing claim on ${config.chainName}...`);
  const tx = await igpContract.claim();
  rootLogger.info(`Transaction sent: ${tx.hash}`);

  const receipt = await tx.wait();
  rootLogger.info(
    `Claim successful on ${config.chainName}. Block: ${receipt.blockNumber}`,
  );

  return tx.hash;
}

/**
 * Process a single chain
 */
async function processChain(config: ChainConfig): Promise<void> {
  try {
    rootLogger.info(`Checking ${config.chainName}...`);

    const checkResult = await shouldClaim(config);
    rootLogger.info(
      {
        chain: config.chainName,
        igpBalance: checkResult.igpBalance,
        relayerBalance: checkResult.relayerBalance,
        reason: checkResult.reason,
      },
      `Balance check result`,
    );

    if (checkResult.shouldClaim) {
      const txHash = await executeClaim(config);
      rootLogger.info(
        {
          chain: config.chainName,
          txHash,
        },
        `Claim executed successfully`,
      );
    } else {
      rootLogger.info(
        `Skipping ${config.chainName}: ${checkResult.reason}`,
      );
    }
  } catch (error) {
    const errorMsg =
      error instanceof Error ? error.message : String(error);
    rootLogger.error(
      {
        chain: config.chainName,
        error: errorMsg,
      },
      `Error processing chain`,
    );
  }
}

/**
 * Main function - runs in daemon mode if INTERVAL is set
 */
async function main() {
  const configs = parseChainConfigs();
  rootLogger.info(`Loaded ${configs.length} chain configuration(s)`);

  const intervalSeconds = parseInt(process.env.INTERVAL || '0', 10);
  const dryRun = process.env.DRY_RUN === 'true';

  if (dryRun) {
    rootLogger.info('DRY RUN mode: will not execute transactions');
  }

  // Process all chains once
  const processAllChains = async () => {
    rootLogger.info('Starting balance check cycle...');
    await Promise.all(configs.map((config) => processChain(config)));
    rootLogger.info('Balance check cycle completed');
  };

  if (intervalSeconds > 0) {
    // Daemon mode: run periodically
    rootLogger.info(
      `Running in daemon mode with ${intervalSeconds}s interval`,
    );
    rootLogger.info('Press Ctrl+C to stop');

    // Run immediately
    await processAllChains();

    // Then run on interval
    setInterval(async () => {
      await processAllChains();
    }, intervalSeconds * 1000);
  } else {
    // One-time execution
    await processAllChains();
    process.exit(0);
  }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
  rootLogger.info('Shutting down...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  rootLogger.info('Shutting down...');
  process.exit(0);
});

main().catch((error) => {
  rootLogger.error({ error }, 'Fatal error');
  process.exit(1);
});

