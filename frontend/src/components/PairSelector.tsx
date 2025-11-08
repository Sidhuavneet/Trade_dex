import { useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Search } from 'lucide-react';
import { Input } from '@/components/ui/input';

interface PairSelectorProps {
  value: string;
  onChange: (pair: string) => void;
}

const AVAILABLE_PAIRS = [
  'SOL/USDC',
  'SOL/USDT',
  'BONK/SOL',
  'JUP/SOL',
  'WIF/SOL',
  'RAY/SOL',
];

export const PairSelector = ({ value, onChange }: PairSelectorProps) => {
  const [searchQuery, setSearchQuery] = useState('');

  const filteredPairs = AVAILABLE_PAIRS.filter((pair) =>
    pair.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="flex items-center gap-2">
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className="w-[180px] bg-secondary border-border">
          <SelectValue placeholder="Select pair" />
        </SelectTrigger>
        <SelectContent className="bg-popover border-border">
          <div className="p-2 border-b border-border">
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search pairs..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-8 bg-secondary border-border"
              />
            </div>
          </div>
          {filteredPairs.map((pair) => (
            <SelectItem
              key={pair}
              value={pair}
              className="hover:bg-secondary cursor-pointer"
            >
              {pair}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
};
