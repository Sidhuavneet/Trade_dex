import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { WalletButton } from './WalletButton';
import { PairSelector } from './PairSelector';
import { Button } from './ui/button';
import { TradesModal } from './TradesModal';
import { PortfolioModal } from './PortfolioModal';
import { History, Briefcase } from 'lucide-react';

interface HeaderProps {
  selectedPair: string;
  onPairChange: (pair: string) => void;
}

export const Header = ({ selectedPair, onPairChange }: HeaderProps) => {
  const [tradesModalOpen, setTradesModalOpen] = useState(false);
  const [portfolioModalOpen, setPortfolioModalOpen] = useState(false);
  const navigate = useNavigate();

  const handleLogoClick = () => {
    navigate('/');
  };

  return (
    <>
      <header className="border-b border-border bg-card">
        <div className="flex items-center justify-between px-6 py-4">
          <div className="flex items-center gap-8">
            <div 
              className="flex items-center gap-3 cursor-pointer hover:opacity-80 transition-opacity"
              onClick={handleLogoClick}
            >
              <div className="w-8 h-8 bg-primary rounded-md flex items-center justify-center">
                <div className="flex flex-col gap-0.5">
                  <div className="w-4 h-0.5 bg-primary-foreground" />
                  <div className="w-4 h-0.5 bg-primary-foreground" />
                  <div className="w-4 h-0.5 bg-primary-foreground" />
                </div>
              </div>
              <h1 className="text-xl font-bold">TRADE</h1>
            </div>
            <PairSelector value={selectedPair} onChange={onPairChange} />
          </div>
          <div className="flex items-center gap-3">
            <Button
              variant="outline"
              onClick={() => setTradesModalOpen(true)}
              className="border-border"
            >
              <History className="w-4 h-4 mr-2" />
              Trades
            </Button>
            <Button
              variant="outline"
              onClick={() => setPortfolioModalOpen(true)}
              className="border-border"
            >
              <Briefcase className="w-4 h-4 mr-2" />
              Portfolio
            </Button>
            <WalletButton />
          </div>
        </div>
      </header>
      
      <TradesModal
        open={tradesModalOpen}
        onOpenChange={setTradesModalOpen}
        pair={selectedPair}
      />
      
      <PortfolioModal
        open={portfolioModalOpen}
        onOpenChange={setPortfolioModalOpen}
      />
    </>
  );
};
