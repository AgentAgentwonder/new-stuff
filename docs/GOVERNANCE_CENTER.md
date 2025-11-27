# Governance Center

The Governance Center provides a comprehensive interface for participating in DAO governance across multiple platforms.

## Supported DAO Platforms

### Realms
- Solana's primary DAO governance platform
- Full support for proposal voting and delegation
- Integration with SPL Governance program

### Tribeca (Govern)
- Protocol-specific governance for DeFi protocols
- Support for vote locking and boosted voting power
- Integration with Tribeca SDK

### Squads
- Multisig-based governance
- Support for program upgrades and treasury management
- Integration with Squads Protocol

### Custom
- Support for custom governance implementations
- Extensible architecture for new platforms

## Features

### Membership Tracking
- Automatic detection of DAO memberships based on token holdings
- Real-time voting power calculation
- Support for delegated voting power
- Membership history and activity tracking

### Proposal Management
- View all active proposals across your DAOs
- Filter and search proposals by DAO, status, or tags
- View proposal details including instructions and discussion links
- Track voting deadlines with urgency indicators

### Voting Actions
- Secure signature-based voting flow
- Vote with Yes/No/Abstain options
- Real-time vote tally updates
- Transaction confirmation and tracking

### Vote Delegation
- Delegate your voting power to trusted addresses
- Set delegation expiration dates
- Revoke delegations at any time
- Track incoming delegations from others

### Proposal Impact Analysis
- AI-powered similarity scoring with historical proposals
- Risk factor identification
- Predicted outcome based on historical data
- Recommended voting based on confidence scores
- Similar proposal comparison

### Reminders & Notifications
- Upcoming deadline tracking
- Urgency-based visual indicators
- Push notifications for important votes
- Customizable reminder settings

## Security & Permissions

### Wallet Integration
- Seamless integration with connected wallets
- Support for hardware wallets (Ledger)
- Message signing for off-chain voting
- Transaction signing for on-chain voting

### Permission Scopes
The governance system requires the following permissions:

1. **Read Permissions**:
   - Query token account balances
   - Read governance account state
   - View proposal details
   - Check voting records

2. **Write Permissions**:
   - Sign vote messages (off-chain voting)
   - Submit vote transactions (on-chain voting)
   - Delegate voting power
   - Revoke delegations

3. **No Permissions Required**:
   - View public proposals
   - Read DAO information
   - Browse governance platforms

### Security Best Practices
- All vote transactions are reviewed before signing
- Clear display of voting power being used
- Confirmation step for all governance actions
- Audit trail for all votes and delegations

## API Reference

### Backend Commands

#### `sync_governance_memberships`
Syncs DAO memberships for a wallet address.
```rust
sync_governance_memberships(wallet_address: String) -> Vec<DAOMembership>
```

#### `get_all_active_governance_proposals`
Fetches all active proposals for user's DAOs.
```rust
get_all_active_governance_proposals(wallet_address: String) -> Vec<GovernanceProposal>
```

#### `submit_signed_vote`
Submits a vote with signature verification.
```rust
submit_signed_vote(
    proposal_id: String,
    wallet_address: String,
    vote_choice: VoteChoice,
    signature: String
) -> VoteRecord
```

#### `delegate_governance_votes`
Delegates voting power to another address.
```rust
delegate_governance_votes(
    dao_id: String,
    delegator: String,
    delegate: String,
    voting_power: f64,
    expires_at: Option<i64>
) -> DelegationRecord
```

#### `analyze_governance_proposal`
Analyzes proposal impact using historical data.
```rust
analyze_governance_proposal(proposal_id: String) -> ProposalImpactAnalysis
```

#### `get_governance_summary`
Gets governance overview for a wallet.
```rust
get_governance_summary(wallet_address: String) -> GovernanceSummary
```

### Frontend Components

#### `<Governance />`
Main governance page component with tabbed interface.

#### `<VotingModal />`
Modal for casting votes on proposals with signature flow.

#### `<ProposalImpactModal />`
Modal displaying AI-powered impact analysis.

#### `<DelegationModal />`
Modal for delegating voting power.

#### `<ProposalList />`
List view of governance proposals with filtering.

#### `<DAOMembershipList />`
Grid view of user's DAO memberships.

#### `<UpcomingDeadlines />`
Priority list of upcoming voting deadlines.

## Data Types

### DAOMembership
Represents membership in a DAO.
- `daoId`: Unique identifier for the DAO
- `daoName`: Display name of the DAO
- `platform`: Governance platform (realms, tribeca, squads, custom)
- `votingPower`: Current voting power
- `delegatedTo`: Address votes are delegated to (if any)
- `delegatedFrom`: Addresses that have delegated to this wallet

### GovernanceProposal
Represents a governance proposal.
- `proposalId`: Unique identifier
- `title`: Proposal title
- `description`: Full description
- `status`: Current status (draft, active, succeeded, etc.)
- `votingEndsAt`: Deadline timestamp
- `yesVotes`, `noVotes`, `abstainVotes`: Current vote tallies
- `quorumRequired`: Minimum votes needed
- `instructions`: On-chain instructions to execute

### ProposalImpactAnalysis
AI-powered analysis of proposal impact.
- `historicalSimilarityScore`: Overall similarity to past proposals
- `similarProposals`: List of similar historical proposals
- `predictedOutcome`: Predicted result (succeeded/defeated)
- `confidence`: Confidence level (0-1)
- `riskFactors`: Identified risks
- `recommendedVote`: AI recommendation (yes/no/abstain)

## Testing

### Backend Tests
Run governance backend tests:
```bash
cd src-tauri
cargo test governance::
```

Key test coverage:
- Membership syncing
- Proposal fetching
- Vote submission
- Delegation flow
- Impact analysis

### Frontend Tests
Run governance UI tests:
```bash
npm test -- governance
```

Key test coverage:
- Component rendering
- Vote submission flow
- Modal interactions
- Data loading states

## Development

### Adding New DAO Platforms

1. Add platform to `DAOPlatform` enum in `types.rs`
2. Implement data fetching in `manager.rs`
3. Add platform-specific logic in `fetch_dao_proposals`
4. Update frontend types in `types/governance.ts`
5. Add platform icon/branding in components

### Extending Impact Analysis

The impact analysis system can be extended by:
1. Adding new similarity metrics in `find_similar_proposals`
2. Implementing ML-based outcome prediction
3. Adding more risk factors in `analyze_proposal_impact`
4. Integrating with external data sources

## Troubleshooting

### Common Issues

**Issue**: Memberships not appearing
- **Solution**: Ensure wallet has governance token holdings
- Check token account permissions

**Issue**: Voting fails
- **Solution**: Verify voting period is active
- Check wallet has sufficient voting power
- Ensure transaction signature is valid

**Issue**: Delegations not working
- **Solution**: Verify delegate address is valid
- Check delegation hasn't expired
- Confirm on-chain delegation transaction succeeded

## Roadmap

Future enhancements planned:
- [ ] Support for more DAO platforms (Governance v2, Mango v4)
- [ ] Advanced filtering and search
- [ ] Governance calendar view
- [ ] Vote history analytics
- [ ] Reputation scoring for delegates
- [ ] Proposal creation interface
- [ ] Multi-wallet voting
- [ ] Governance notifications via email/SMS
