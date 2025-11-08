import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';

interface PortfolioModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const PortfolioModal = ({ open, onOpenChange }: PortfolioModalProps) => {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl">
        <DialogHeader>
          <DialogTitle>Portfolio</DialogTitle>
          <DialogDescription>
            Your trading portfolio and holdings
          </DialogDescription>
        </DialogHeader>
        
        <div className="py-8">
          <p className="text-center text-muted-foreground">
            Portfolio feature coming soon...
          </p>
        </div>
      </DialogContent>
    </Dialog>
  );
};

