#!/usr/bin/env node

/**
 * CTO Environment Setup Script
 * Automatically configures environment variables for the memecoin trading application
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

class EnvSetup {
  constructor() {
    this.configPath = path.join(__dirname, '..', '..', '.cto', 'config.json');
    this.envPath = path.join(__dirname, '..', '..', '.env.local');
    this.config = this.loadConfig();
  }

  loadConfig() {
    try {
      return JSON.parse(fs.readFileSync(this.configPath, 'utf8'));
    } catch (error) {
      console.error('Failed to load CTO config:', error.message);
      process.exit(1);
    }
  }

  validateApiKey(key, name) {
    const config = this.config.environment.required_vars[name];
    if (!config) return true;

    if (config.validation) {
      const regex = new RegExp(config.validation);
      if (!regex.test(key)) {
        console.error(`❌ Invalid ${name}: ${key}`);
        console.error(`Expected format: ${config.validation}`);
        return false;
      }
    }

    if (config.sensitive) {
      console.log(`✅ ${name} is valid (sensitive - value hidden)`);
    } else {
      console.log(`✅ ${name} is valid: ${key}`);
    }

    return true;
  }

  promptForVariable(name) {
    const config = this.config.environment.required_vars[name];
    if (!config) return null;

    const readline = require('readline').createInterface({
      input: process.stdin,
      output: process.stdout
    });

    return new Promise((resolve) => {
      const prompt = config.sensitive 
        ? `Enter ${config.description} (sensitive): `
        : `Enter ${config.description} (${config.default || 'no default'}): `;

      readline.question(prompt, (answer) => {
        readline.close();
        
        const value = answer.trim() || config.default;
        if (!value && config.required) {
          console.error(`❌ ${name} is required`);
          process.exit(1);
        }

        if (value && !this.validateApiKey(value, name)) {
          process.exit(1);
        }

        resolve(value);
      });
    });
  }

  async setupEnvironment() {
    console.log('🚀 Setting up CTO Environment for Memecoin Trader\n');

    const envVars = {};
    const requiredVars = Object.keys(this.config.environment.required_vars);

    for (const varName of requiredVars) {
      const config = this.config.environment.required_vars[varName];
      const existingValue = process.env[varName];

      if (existingValue) {
        if (this.validateApiKey(existingValue, varName)) {
          envVars[varName] = existingValue;
        } else {
          console.log(`⚠️  Using existing ${varName} but it may be invalid`);
          envVars[varName] = existingValue;
        }
      } else if (fs.existsSync(this.envPath)) {
        const existingEnv = fs.readFileSync(this.envPath, 'utf8');
        const match = existingEnv.match(new RegExp(`^${varName}=(.+)$`, 'm'));
        if (match) {
          const value = match[1].trim();
          if (this.validateApiKey(value, varName)) {
            envVars[varName] = value;
            console.log(`✅ Using existing ${varName} from .env.local`);
          }
        }
      }

      if (!envVars[varName]) {
        envVars[varName] = await this.promptForVariable(varName);
      }
    }

    // Write environment file
    const envContent = Object.entries(envVars)
      .map(([key, value]) => `${key}=${value}`)
      .join('\n');

    fs.writeFileSync(this.envPath, envContent);
    console.log(`\n✅ Environment written to: ${this.envPath}`);

    // Set permissions for sensitive files
    if (envVars.WALLET_PRIVATE_KEY) {
      fs.chmodSync(this.envPath, 0o600);
      console.log('🔒 Set secure permissions for .env.local');
    }

    console.log('\n🎯 Setup complete! You can now run:');
    console.log('   npm run dev        # Start development');
    console.log('   npm run build     # Build for production');
    console.log('   npm run test      # Run tests');
  }

  generateSecureToken() {
    return crypto.randomBytes(32).toString('hex');
  }
}

// Run setup if called directly
if (require.main === module) {
  const setup = new EnvSetup();
  setup.setupEnvironment().catch(console.error);
}

module.exports = EnvSetup;
