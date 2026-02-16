'use client';

import { useEffect, useMemo, useState } from 'react';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Switch } from '@/components/ui/switch';
import { useContractRiskStore } from '@/store/contractRiskStore';

const trustLevelConfig = {
  safe: { label: 'Safe', color: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' },
  caution: { label: 'Caution', color: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20' },
  danger: { label: 'Danger', color: 'bg-orange-500/10 text-orange-400 border-orange-500/20' },
  critical: { label: 'Critical', color: 'bg-red-500/10 text-red-400 border-red-500/20' },
};

const severityBadge: Record<string, string> = {
  low: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
  medium: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20',
  high: 'bg-orange-500/10 text-orange-400 border-orange-500/20',
  critical: 'bg-red-500/10 text-red-400 border-red-500/20',
};

export default function ContractRiskPage() {
  const [contractAddress, setContractAddress] = useState('');
  const {
    assessment,
    events,
    monitoredContracts,
    emergencyHaltActive,
    isLoading,
    error,
    assessContract,
    loadRiskEvents,
    monitorContract,
    unmonitorContract,
    refreshMonitoredContracts,
    loadMonitoredContracts,
    fetchEmergencyHalt,
    setEmergencyHalt,
  } = useContractRiskStore();

  useEffect(() => {
    loadMonitoredContracts();
    fetchEmergencyHalt();
  }, [loadMonitoredContracts, fetchEmergencyHalt]);

  useEffect(() => {
    if (!assessment?.address) return;
    loadRiskEvents(assessment.address);
  }, [assessment?.address, loadRiskEvents]);

  useEffect(() => {
    if (!monitoredContracts.length) return;
    const interval = window.setInterval(() => {
      refreshMonitoredContracts();
    }, 30000);
    return () => window.clearInterval(interval);
  }, [monitoredContracts.length, refreshMonitoredContracts]);

  const trustLevel = useMemo(() => {
    if (!assessment) return 'caution';
    const score = assessment.securityScore;
    if (score >= 80) return 'safe';
    if (score >= 60) return 'caution';
    if (score >= 40) return 'danger';
    return 'critical';
  }, [assessment]);

  const recommendedActions = useMemo(() => {
    if (!assessment) return [];
    const actions = [] as string[];
    if (assessment.securityScore < 50) {
      actions.push('Reduce position size and tighten stop-loss');
    }
    if (!assessment.verificationData.codeVerified) {
      actions.push('Avoid trading until contract is verified');
    }
    if (!assessment.verificationData.liquidityLocked) {
      actions.push('Wait for liquidity lock confirmation');
    }
    if (assessment.marketMetrics.bidAskSpreadPercent > 5) {
      actions.push('Avoid illiquid markets with wide spreads');
    }
    if (!actions.length) {
      actions.push('Proceed with normal risk controls and monitoring');
    }
    return actions;
  }, [assessment]);

  const isMonitored = assessment?.address
    ? monitoredContracts.includes(assessment.address)
    : false;

  const riskScore = assessment ? Math.round(assessment.riskScore * 100) : 0;

  const handleAssess = async () => {
    if (!contractAddress.trim()) return;
    await assessContract(contractAddress.trim());
  };

  const handleToggleMonitor = async () => {
    if (!assessment?.address) return;
    if (isMonitored) {
      await unmonitorContract(assessment.address);
    } else {
      await monitorContract(assessment.address);
    }
  };

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Contract Risk Assessment</h1>
        <p className="text-muted-foreground mt-1">
          Comprehensive smart contract safety checks for memecoin trading.
        </p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Analyze Contract</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-col gap-3 md:flex-row md:items-center">
            <Input
              placeholder="Enter Solana contract address"
              value={contractAddress}
              onChange={event => setContractAddress(event.target.value)}
              className="flex-1"
            />
            <Button onClick={handleAssess} disabled={isLoading}>
              {isLoading ? 'Analyzing...' : 'Run Assessment'}
            </Button>
            <Button
              variant="outline"
              onClick={handleToggleMonitor}
              disabled={!assessment}
            >
              {isMonitored ? 'Stop Monitoring' : 'Monitor Contract'}
            </Button>
          </div>
          {error && <p className="text-sm text-red-400">{error}</p>}
        </CardContent>
      </Card>

      <div className="grid gap-6 lg:grid-cols-[2fr,1fr]">
        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle>Risk Overview</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            {assessment ? (
              <>
                <div className="flex flex-wrap items-center gap-3">
                  <Badge className={trustLevelConfig[trustLevel].color} variant="outline">
                    {trustLevelConfig[trustLevel].label}
                  </Badge>
                  <Badge variant="secondary">{assessment.verificationStatus}</Badge>
                  <span className="text-sm text-muted-foreground">
                    Updated {new Date(assessment.assessedAt).toLocaleString()}
                  </span>
                </div>
                <div className="space-y-3">
                  <div className="flex items-center justify-between text-sm">
                    <span>Security Score</span>
                    <span className="font-semibold">{Math.round(assessment.securityScore)}</span>
                  </div>
                  <Progress value={assessment.securityScore} className="h-2" />
                  <div className="flex items-center justify-between text-sm">
                    <span>Risk Score</span>
                    <span className="font-semibold">{riskScore}</span>
                  </div>
                  <Progress value={riskScore} className="h-2" />
                </div>
                <Separator />
                <div>
                  <h3 className="text-sm font-semibold text-foreground mb-2">Recommended Actions</h3>
                  <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
                    {recommendedActions.map(action => (
                      <li key={action}>{action}</li>
                    ))}
                  </ul>
                </div>
              </>
            ) : (
              <p className="text-sm text-muted-foreground">
                Run a contract assessment to view detailed risk metrics and alerts.
              </p>
            )}
          </CardContent>
        </Card>

        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle>Emergency Controls</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-foreground">Trading Halt</p>
                <p className="text-xs text-muted-foreground">
                  Instantly block all trades when activated.
                </p>
              </div>
              <Switch
                checked={emergencyHaltActive}
                onCheckedChange={setEmergencyHalt}
              />
            </div>
            <div className="rounded-md border border-border p-3 text-xs text-muted-foreground">
              {emergencyHaltActive
                ? 'Emergency halt is active. All trades will be blocked by the SafetyEngine.'
                : 'Emergency halt is inactive. Trading will proceed with standard safety checks.'}
            </div>
            <div>
              <p className="text-xs uppercase tracking-wide text-muted-foreground">
                Monitored contracts
              </p>
              <div className="mt-2 space-y-1 text-xs text-muted-foreground">
                {monitoredContracts.length ? (
                  monitoredContracts.map(address => (
                    <div key={address} className="truncate">
                      {address}
                    </div>
                  ))
                ) : (
                  <p>No active monitoring.</p>
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle>Honeypot Detection</CardTitle>
          </CardHeader>
          <CardContent>
            {assessment ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Indicator</TableHead>
                    <TableHead>Severity</TableHead>
                    <TableHead>Status</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {assessment.honeypotIndicators.map(indicator => (
                    <TableRow key={indicator.category}>
                      <TableCell>{indicator.description}</TableCell>
                      <TableCell>
                        <Badge
                          variant="outline"
                          className={severityBadge[indicator.severity]}
                        >
                          {indicator.severity}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        {indicator.triggered ? (
                          <Badge className="bg-red-500/10 text-red-400 border-red-500/20" variant="outline">
                            Flagged
                          </Badge>
                        ) : (
                          <Badge className="bg-emerald-500/10 text-emerald-400 border-emerald-500/20" variant="outline">
                            Clear
                          </Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <p className="text-sm text-muted-foreground">
                Honeypot detection metrics will appear after assessment.
              </p>
            )}
          </CardContent>
        </Card>

        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle>Market Microstructure</CardTitle>
          </CardHeader>
          <CardContent>
            {assessment ? (
              <div className="space-y-3 text-sm text-muted-foreground">
                <div className="flex justify-between">
                  <span>Bid-Ask Spread</span>
                  <span>{assessment.marketMetrics.bidAskSpreadPercent.toFixed(2)}%</span>
                </div>
                <div className="flex justify-between">
                  <span>Liquidity Depth</span>
                  <span>${assessment.marketMetrics.liquidityDepthUsd.toFixed(0)}</span>
                </div>
                <div className="flex justify-between">
                  <span>24h Volume</span>
                  <span>${assessment.marketMetrics.volume24hUsd.toFixed(0)}</span>
                </div>
                <div className="flex justify-between">
                  <span>Wash Trading Score</span>
                  <span>{assessment.marketMetrics.washTradingScore.toFixed(2)}</span>
                </div>
                <div className="flex justify-between">
                  <span>Price Manipulation Score</span>
                  <span>{assessment.marketMetrics.priceManipulationScore.toFixed(2)}</span>
                </div>
                <div className="flex justify-between">
                  <span>Market Cap</span>
                  <span>${assessment.marketMetrics.marketCapUsd.toFixed(0)}</span>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                Market structure validation will appear after assessment.
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle>Risk Factors</CardTitle>
          </CardHeader>
          <CardContent>
            {assessment ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Category</TableHead>
                    <TableHead>Description</TableHead>
                    <TableHead>Severity</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {assessment.riskFactors.map(factor => (
                    <TableRow key={`${factor.category}-${factor.description}`}>
                      <TableCell className="font-medium">{factor.category}</TableCell>
                      <TableCell>{factor.description}</TableCell>
                      <TableCell>
                        <Badge
                          variant="outline"
                          className={severityBadge[factor.severity]}
                        >
                          {factor.severity}
                        </Badge>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <p className="text-sm text-muted-foreground">
                Risk factor analysis will appear after assessment.
              </p>
            )}
          </CardContent>
        </Card>

        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle>Risk Alerts</CardTitle>
          </CardHeader>
          <CardContent>
            {events.length ? (
              <div className="space-y-3 text-sm">
                {events.map(event => (
                  <div key={event.id} className="border border-border rounded-lg p-3">
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-foreground">{event.eventType}</span>
                      <Badge
                        variant="outline"
                        className={severityBadge[event.severity] ?? 'border-border'}
                      >
                        {event.severity}
                      </Badge>
                    </div>
                    <p className="text-muted-foreground mt-1">{event.description}</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      {new Date(event.timestamp).toLocaleString()}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                No risk alerts yet. High-severity events will appear here.
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
