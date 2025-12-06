import React from 'react';
import { Link } from 'react-router-dom';
import { Music } from 'lucide-react';

export const Navbar: React.FC = () => {
  return (
    <nav className="bg-surface border-b border-gray-800 hidden md:block">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <Link to="/" className="flex items-center gap-2 text-xl font-bold">
            <Music className="text-primary" size={28} />
            <span>Last.fm Dating</span>
          </Link>
          
          <div className="flex items-center gap-6">
            <Link to="/discover" className="hover:text-primary transition-colors">
              Discover
            </Link>
            <Link to="/matches" className="hover:text-primary transition-colors">
              Matches
            </Link>
            <Link to="/profile" className="hover:text-primary transition-colors">
              Profile
            </Link>
          </div>
        </div>
      </div>
    </nav>
  );
};
