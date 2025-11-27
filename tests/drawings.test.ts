import { describe, it, expect, beforeEach } from 'vitest';
import { useDrawingStore } from '../src/store/drawingStore';
import type { DrawingObject } from '../src/types/drawings';

// Reset store between tests
beforeEach(() => {
  useDrawingStore.setState({
    drawings: [],
    templates: [],
    activeTool: null,
    activeStyle: useDrawingStore.getState().activeStyle,
    selectedDrawingId: null,
    isLoading: false,
    error: null,
  });
});

describe('Drawing Store', () => {
  it('should add drawing', () => {
    const { addDrawing, drawings } = useDrawingStore.getState();

    const newDrawing = addDrawing({
      userId: 'user1',
      symbol: 'SOL',
      tool: 'trendline',
      points: [
        { x: 10, y: 20, timestamp: Date.now(), price: 100 },
        { x: 50, y: 70, timestamp: Date.now(), price: 110 },
      ],
      style: useDrawingStore.getState().activeStyle,
      locked: false,
      hidden: false,
    });

    expect(drawings).toHaveLength(1);
    expect(newDrawing.symbol).toBe('SOL');
  });

  it('should update drawing', () => {
    const { addDrawing, updateDrawing } = useDrawingStore.getState();
    const drawing = addDrawing({
      userId: 'user1',
      symbol: 'SOL',
      tool: 'trendline',
      points: [
        { x: 10, y: 20 },
        { x: 50, y: 70 },
      ],
      style: useDrawingStore.getState().activeStyle,
      locked: false,
      hidden: false,
    });

    updateDrawing(drawing.id, {
      points: [
        { x: 15, y: 25 },
        { x: 55, y: 75 },
      ],
    } as Partial<DrawingObject>);

    const updated = useDrawingStore.getState().drawings.find(d => d.id === drawing.id);
    expect(updated?.points[0].x).toBe(15);
  });

  it('should remove drawing', () => {
    const { addDrawing, removeDrawing } = useDrawingStore.getState();
    const drawing = addDrawing({
      userId: 'user1',
      symbol: 'SOL',
      tool: 'trendline',
      points: [
        { x: 10, y: 20 },
        { x: 50, y: 70 },
      ],
      style: useDrawingStore.getState().activeStyle,
      locked: false,
      hidden: false,
    });

    removeDrawing(drawing.id);
    expect(useDrawingStore.getState().drawings).toHaveLength(0);
  });

  it('should duplicate drawing with offset points', () => {
    const { addDrawing, duplicateDrawing } = useDrawingStore.getState();
    const drawing = addDrawing({
      userId: 'user1',
      symbol: 'SOL',
      tool: 'trendline',
      points: [
        { x: 10, y: 20 },
        { x: 50, y: 70 },
      ],
      style: useDrawingStore.getState().activeStyle,
      locked: false,
      hidden: false,
    });

    const duplicate = duplicateDrawing(drawing.id);
    expect(duplicate).not.toBeNull();
    expect(duplicate?.points[0].x).toBe(20);
    expect(duplicate?.points[0].y).toBe(30);
  });
});
