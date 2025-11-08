import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { TradesTable } from './TradesTable';

interface TabsPanelProps {
  pair: string;
}

export const TabsPanel = ({ pair }: TabsPanelProps) => {
  return (
    <div className="h-full flex flex-col">
      <TradesTable pair={pair} />
    </div>
  );
};
