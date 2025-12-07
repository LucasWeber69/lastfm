import React from 'react';
import { Link } from 'react-router-dom';
import { Music } from 'lucide-react';

export const Navbar: React.FC = () => {
  return (
    <nav className="bg-surface/70 backdrop-blur-lg border-b border-border/50 hidden md:block sticky top-0 z-50">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <Link to="/" className="flex items-center gap-2 text-xl font-bold hover:scale-105 transition-transform">
            <Music className="text-primary" size={28} />
            <span className="gradient-text">Last.fm Dating</span>
          </Link>
          
          <div className="flex items-center gap-6">
            <Link to="/discover" className="hover:text-primary transition-colors font-medium">
              Discover
            </Link>
            <Link to="/matches" className="hover:text-primary transition-colors font-medium">
              Matches
            </Link>
            <Link to="/profile" className="hover:text-primary transition-colors font-medium">
              Profile
            </Link>
          </div>
        </div>
      </div>
    </nav>
  );
};
