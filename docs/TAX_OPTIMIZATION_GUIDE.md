# Tax Optimization Engine Guide

## Overview

The Tax Optimization Engine provides comprehensive tax planning capabilities for cryptocurrency portfolios, including:

- **Capital Gains Calculations**: Accurate short-term and long-term capital gains tracking
- **Wash Sale Detection**: Automatic detection of wash sales with recommendations
- **Multi-Jurisdiction Support**: Pre-configured tax rules for US, UK, Germany, and Australia
- **Tax-Loss Harvesting**: AI-powered recommendations for optimizing tax liability
- **Export Capabilities**: One-click export to CoinTracker, Koinly, and CSV formats

## Features

### 1. Capital Gains Tracking

The engine automatically calculates:
- Short-term vs long-term capital gains based on jurisdiction rules
- Net realized gains/losses for the tax year
- Effective tax rate projections
- Unrealized gains and losses

### 2. Wash Sale Detection

Monitors transactions to detect potential wash sales:
- 30-day wash sale period tracking (configurable by jurisdiction)
- Severity classification (High/Medium/Low)
- Automatic calculation of disallowed losses
- Recommendations for alternative assets

### 3. Tax-Loss Harvesting

Smart recommendations for reducing tax liability:
- Priority-based recommendations (Critical/High/Medium/Low)
- Calculated tax savings estimates
- Wash sale risk assessment
- Alternative asset suggestions
- Holding period optimization

### 4. Multi-Jurisdiction Support

Pre-configured tax rules for:

#### United States (Federal)
- Short-term rate: 37%
- Long-term rate: 20%
- Holding period: 365 days
- Wash sale period: 30 days
- Capital loss limit: $3,000/year

#### United Kingdom
- Capital gains rate: 20%
- Holding period: N/A (all gains taxed equally)
- Annual allowance: £12,300
- Wash sale period: 30 days

#### Germany
- Short-term rate: 45%
- Long-term rate: 0% (after 1 year)
- Holding period: 365 days
- No wash sale rules
- Reporting threshold: €600

#### Australia
- Short-term rate: 45%
- Long-term rate: 22.5% (50% discount after 1 year)
- Holding period: 365 days
- Tax year: July 1 - June 30

## Usage

### Frontend Integration

```tsx
import { TaxCenter } from './components/portfolio/TaxCenter';

function App() {
  return <TaxCenter />;
}
```

### API Endpoints

#### Get Tax Center Summary
```typescript
const summary = await invoke('get_tax_center_summary', {
  taxYear: 2024
});
```

#### Update Tax Settings
```typescript
await invoke('update_tax_settings', {
  settings: {
    jurisdiction: {
      code: 'US',
      // ... jurisdiction details
    },
    taxYear: 2024,
    enableWashSaleDetection: true,
    enableTaxLossHarvesting: true,
  }
});
```

#### Export Tax Report
```typescript
const report = await invoke('export_tax_center_report', {
  format: 'cointracker', // or 'koinly', 'csv'
  params: {
    taxYear: 2024
  }
});
```

## Backend Architecture

### Module Structure

```
src-tauri/src/tax/
├── mod.rs              # Main module and Tauri commands
├── types.rs            # Type definitions
├── calculator.rs       # Capital gains calculations
├── wash_sale.rs        # Wash sale detection
├── harvesting.rs       # Tax-loss harvesting logic
├── jurisdiction.rs     # Jurisdiction management
└── exports.rs          # Export formatters
```

### Key Components

#### TaxCalculator
Handles all capital gains calculations:
- Cost basis tracking
- Gain/loss computation
- Tax rate application
- Quarterly estimates

#### WashSaleDetector
Monitors for wash sales:
- Transaction pattern analysis
- 30-day window tracking
- Alternative asset suggestions
- Severity classification

#### TaxLossHarvester
Generates harvesting recommendations:
- Unrealized loss identification
- Tax savings calculation
- Priority ranking
- Wash sale risk assessment

#### JurisdictionManager
Manages tax jurisdiction configurations:
- Secure storage in keystore
- Jurisdiction validation
- Pre-configured tax rules
- Custom jurisdiction support

## Security

### Jurisdiction Data Storage

Jurisdiction configurations are stored securely in the keystore using AES-256-GCM encryption:

```rust
let keystore = Keystore::initialize(&app_handle)?;
let manager = JurisdictionManager::new();

// Store jurisdiction securely
manager.save_jurisdiction(&keystore, "user_id", &jurisdiction)?;

// Retrieve jurisdiction
let jurisdiction = manager.load_jurisdiction(&keystore, "user_id")?;
```

### Data Privacy

- All tax calculations are performed locally
- No sensitive tax data is transmitted to external services
- Jurisdiction settings are encrypted at rest
- Export files are generated locally

## Legal Disclaimer

**IMPORTANT**: This tax optimization engine is provided for educational and informational purposes only. It should NOT be considered as professional tax, legal, or financial advice.

- Tax laws vary by jurisdiction and are subject to frequent changes
- Individual tax situations can be complex and unique
- Cryptocurrency tax treatment is evolving and may differ from traditional assets
- Always consult with a qualified tax professional or CPA
- This tool does not guarantee accuracy or compliance with tax regulations
- Users are responsible for their own tax reporting and compliance

## Testing

The tax engine includes comprehensive tests:

```bash
# Run all tax tests
cargo test tax

# Run specific test module
cargo test tax::calculator
cargo test tax::wash_sale
cargo test tax::harvesting
```

### Test Coverage

- Capital gains calculations (short-term and long-term)
- Wash sale detection scenarios
- Tax-loss harvesting recommendations
- Jurisdiction configuration
- Export format generation

## Best Practices

### 1. Regular Reviews
- Review tax projections monthly
- Monitor wash sale warnings
- Check harvesting opportunities quarterly

### 2. Year-End Planning
- Review tax liability 60-90 days before year-end
- Execute harvesting strategies strategically
- Consider timing of large transactions

### 3. Record Keeping
- Export reports monthly for backup
- Maintain transaction records
- Document cost basis for all acquisitions

### 4. Professional Consultation
- Consult with a tax professional annually
- Review jurisdiction settings with advisor
- Validate calculations before filing

## Customization

### Adding Custom Jurisdictions

```rust
let custom_jurisdiction = TaxJurisdiction {
    code: "CUSTOM".to_string(),
    name: "Custom Jurisdiction".to_string(),
    short_term_rate: 0.30,
    long_term_rate: 0.15,
    holding_period_days: 365,
    wash_sale_period_days: 30,
    capital_loss_limit: Some(3000.0),
    tax_year_start: "01-01".to_string(),
    requires_reporting_threshold: Some(10.0),
    supports_like_kind_exchange: false,
    crypto_specific_rules: HashMap::new(),
};
```

### Custom Tax Rules

You can add custom crypto-specific rules:

```rust
let mut crypto_rules = HashMap::new();
crypto_rules.insert("staking_income_taxable".to_string(), json!(true));
crypto_rules.insert("defi_yield_rate".to_string(), json!(0.40));
jurisdiction.crypto_specific_rules = crypto_rules;
```

## Troubleshooting

### Common Issues

**Issue**: Wash sale not detected
- Verify transaction dates are within 30-day window
- Check that asset symbols match exactly
- Ensure recent transactions are loaded

**Issue**: Incorrect tax rates
- Verify jurisdiction is correctly selected
- Check for custom tax rate overrides
- Update jurisdiction if tax laws changed

**Issue**: Export fails
- Check that tax lots have disposed dates
- Verify tax year has completed transactions
- Ensure sufficient disk space

## API Reference

### Commands

#### `get_tax_center_summary`
Returns comprehensive tax summary including projections, warnings, and recommendations.

**Parameters:**
- `taxYear: Option<i32>` - Tax year to analyze (defaults to current year)

**Returns:** `TaxCenterSummary`

#### `update_tax_settings`
Updates user tax settings and jurisdiction.

**Parameters:**
- `settings: TaxSettings` - New tax settings

**Returns:** `TaxSettings`

#### `export_tax_center_report`
Exports tax report in specified format.

**Parameters:**
- `format: String` - Export format ("cointracker", "koinly", "csv")
- `params: TaxReportParams` - Report parameters including tax year

**Returns:** `TaxExportFormat`

## Contributing

When contributing to the tax engine:

1. Add comprehensive tests for new features
2. Update documentation with tax law references
3. Consider international tax law variations
4. Include legal disclaimers for new features
5. Validate calculations with tax professionals

## Resources

- [IRS Cryptocurrency Guidance](https://www.irs.gov/businesses/small-businesses-self-employed/virtual-currencies)
- [UK Crypto Tax Guidelines](https://www.gov.uk/government/publications/tax-on-cryptoassets)
- [German Crypto Tax Information](https://www.bundesfinanzministerium.de/)
- [Australian Crypto Tax Guidelines](https://www.ato.gov.au/General/Gen/Tax-treatment-of-crypto-currencies-in-Australia/)

## Support

For issues or questions:
- Review the troubleshooting section
- Check test files for usage examples
- Consult with a tax professional for specific tax advice
- File issues on the project repository (technical issues only)

## Version History

### Version 1.0.0
- Initial release with US, UK, DE, AU jurisdictions
- Capital gains tracking
- Wash sale detection
- Tax-loss harvesting
- CoinTracker/Koinly exports
