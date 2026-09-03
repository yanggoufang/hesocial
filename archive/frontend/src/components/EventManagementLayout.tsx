import React from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Briefcase, LayoutGrid, Map } from 'lucide-react';

interface EventManagementLayoutProps {
  children: React.ReactNode;
}

const navItems = [
  { name: 'Events', path: '/event-mgmt', icon: Briefcase },
  { name: 'Categories', path: '/event-mgmt/categories', icon: LayoutGrid },
  { name: 'Venues', path: '/event-mgmt/venues', icon: Map },
];

const getTitle = (pathname: string): string => {
  if (pathname.includes('categories')) return 'Category Management';
  if (pathname.includes('venues')) return 'Venue Management';
  return 'Event Dashboard';
};

const EventManagementLayout: React.FC<EventManagementLayoutProps> = ({ children }) => {
  const location = useLocation();
  const title = getTitle(location.pathname);

  return (
    <div className="p-4 sm:p-6 lg:p-8 bg-luxury-midnight-black min-h-screen text-luxury-platinum">
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="mb-8"
      >
        <h1 className="text-3xl font-bold text-luxury-gold font-luxury tracking-wider">{title}</h1>
        <p className="text-luxury-platinum/60 mt-1">Manage all aspects of your events, categories, and venues.</p>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.1 }}
        className="luxury-glass p-2 rounded-xl mb-8"
      >
        <nav className="flex items-center space-x-2 sm:space-x-4">
          {navItems.map((item) => (
            <NavLink
              key={item.name}
              to={item.path}
              end={item.path === '/event-mgmt'} // Ensures exact match for parent route
              className={({ isActive }) =>
                `flex-1 sm:flex-initial flex items-center justify-center px-3 py-2 rounded-lg transition-all duration-300 text-sm sm:text-base font-medium ` +
                (isActive
                  ? 'bg-luxury-gold/20 text-luxury-gold shadow-inner'
                  : 'text-luxury-platinum/70 hover:bg-luxury-gold/10 hover:text-luxury-platinum')
              }
            >
              <item.icon className="w-4 h-4 mr-2" />
              <span>{item.name}</span>
            </NavLink>
          ))}
        </nav>
      </motion.div>

      <main>{children}</main>
    </div>
  );
};

export default EventManagementLayout;
