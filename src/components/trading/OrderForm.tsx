import { useState, useCallback, useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useTradingStore } from '@/store/tradingStore';
import { useWalletStore } from '@/store/walletStore';
import { useShallow } from '@/store/createBoundStore';
import { AlertCircle } from 'lucide-react';
import type { CreateOrderRequest } from '@/types';

export function OrderForm() {
  const tradingSelector = useCallback(
    (state: ReturnType<typeof useTradingStore.getState>) => ({
      createOrder: state.createOrder,
      isLoading: state.isLoading,
      error: state.error,
    }),
    []
  );
  const { createOrder, isLoading, error } = useTradingStore(tradingSelector, useShallow);

  const walletSelector = useCallback(
    (state: ReturnType<typeof useWalletStore.getState>) => ({
      activeAccount: state.activeAccount,
    }),
    []
  );
  const { activeAccount } = useWalletStore(walletSelector, useShallow);

  const [orderType, setOrderType] = useState<'market' | 'limit' | 'stop_limit'>('market');
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [inputMint, setInputMint] = useState('');
  const [outputMint, setOutputMint] = useState('');
  const [inputSymbol, setInputSymbol] = useState('SOL');
  const [outputSymbol, setOutputSymbol] = useState('USDC');
  const [amount, setAmount] = useState('');
  const [limitPrice, setLimitPrice] = useState('');
  const [stopPrice, setStopPrice] = useState('');

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();

      if (!activeAccount) {
        alert('Please connect your wallet first');
        return;
      }

      const parsedAmount = parseFloat(amount);
      if (isNaN(parsedAmount) || parsedAmount <= 0) {
        alert('Please enter a valid amount');
        return;
      }

      const request: CreateOrderRequest = {
        orderType,
        side,
        inputMint: inputMint || 'So11111111111111111111111111111111111111112',
        outputMint: outputMint || 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
        inputSymbol,
        outputSymbol,
        amount: parsedAmount,
        slippageBps: 100,
        priorityFeeMicroLamports: 1000,
        walletAddress: activeAccount.publicKey,
      };

      if (orderType === 'limit' && limitPrice) {
        request.limitPrice = parseFloat(limitPrice);
      }

      if (orderType === 'stop_limit' && stopPrice) {
        request.stopPrice = parseFloat(stopPrice);
      }

      try {
        await createOrder(request);
        setAmount('');
        setLimitPrice('');
        setStopPrice('');
      } catch (err) {
        console.error('Failed to create order:', err);
      }
    },
    [
      activeAccount,
      amount,
      createOrder,
      inputMint,
      inputSymbol,
      limitPrice,
      orderType,
      outputMint,
      outputSymbol,
      side,
      stopPrice,
    ]
  );

  const isFormValid = useMemo(() => {
    return activeAccount && amount && parseFloat(amount) > 0;
  }, [activeAccount, amount]);

  return (
    <Card className="bg-card border-border">
      <CardHeader>
        <CardTitle className="text-lg">Create Order</CardTitle>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert variant="destructive" className="mb-4">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid grid-cols-2 gap-2">
            <Button
              type="button"
              variant={side === 'buy' ? 'default' : 'outline'}
              onClick={() => setSide('buy')}
              className={side === 'buy' ? 'bg-accent hover:bg-accent/90' : ''}
            >
              Buy
            </Button>
            <Button
              type="button"
              variant={side === 'sell' ? 'default' : 'outline'}
              onClick={() => setSide('sell')}
              className={side === 'sell' ? 'bg-destructive hover:bg-destructive/90' : ''}
            >
              Sell
            </Button>
          </div>

          <div>
            <Label htmlFor="orderType">Order Type</Label>
            <Select value={orderType} onValueChange={(v: any) => setOrderType(v)}>
              <SelectTrigger id="orderType">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="market">Market</SelectItem>
                <SelectItem value="limit">Limit</SelectItem>
                <SelectItem value="stop_limit">Stop Limit</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <Label htmlFor="inputSymbol">From</Label>
              <Input
                id="inputSymbol"
                value={inputSymbol}
                onChange={e => setInputSymbol(e.target.value)}
                placeholder="SOL"
              />
            </div>
            <div>
              <Label htmlFor="outputSymbol">To</Label>
              <Input
                id="outputSymbol"
                value={outputSymbol}
                onChange={e => setOutputSymbol(e.target.value)}
                placeholder="USDC"
              />
            </div>
          </div>

          <div>
            <Label htmlFor="amount">Amount</Label>
            <Input
              id="amount"
              type="number"
              step="0.000001"
              value={amount}
              onChange={e => setAmount(e.target.value)}
              placeholder="0.00"
            />
          </div>

          {orderType === 'limit' && (
            <div>
              <Label htmlFor="limitPrice">Limit Price</Label>
              <Input
                id="limitPrice"
                type="number"
                step="0.000001"
                value={limitPrice}
                onChange={e => setLimitPrice(e.target.value)}
                placeholder="0.00"
              />
            </div>
          )}

          {orderType === 'stop_limit' && (
            <div>
              <Label htmlFor="stopPrice">Stop Price</Label>
              <Input
                id="stopPrice"
                type="number"
                step="0.000001"
                value={stopPrice}
                onChange={e => setStopPrice(e.target.value)}
                placeholder="0.00"
              />
            </div>
          )}

          <Button
            type="submit"
            className="w-full"
            disabled={!isFormValid || isLoading}
            variant={side === 'buy' ? 'default' : 'destructive'}
          >
            {isLoading ? 'Submitting...' : `${side === 'buy' ? 'Buy' : 'Sell'} ${outputSymbol}`}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
